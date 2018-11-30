//Transctions in zei

// use bulletproofs::RangeProof;
use curve25519_dalek::ristretto::{ CompressedRistretto, RistrettoPoint };
use curve25519_dalek::scalar::Scalar;
use rand::CryptoRng;
use rand::Rng;
use organism_utils::crypto::lockbox::Lockbox;
use organism_utils::helpers::{ be_u8_from_u32, slice_to_fixed32 };
use crate::setup::PublicParams;
use merlin::Transcript;
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use schnorr::PublicKey;
use schnorr::SecretKey;


// A Confidential transaction
// range proof that balance - balance_inc is between (0, val_max)
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
        //this transaction range proof
        //senders updated balance range proof
        pub transaction_range_proof: bulletproofs::RangeProof,
        //transactions Pederson commitment
        pub transaction_commitment: CompressedRistretto,
        //senders updated balance Pederson commitment
        pub sender_updated_balance_commitment: CompressedRistretto,
        //receiver updated commit
        pub receiver_new_commit: CompressedRistretto,
        //lock box
        pub lockbox: Lockbox
}

//helper structure to receive the data for a transaction
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTx {
        pub receiver: PublicKey,
        pub receiver_commit: CompressedRistretto,
        pub transfer_amount: u32,
}


impl Transaction {

        //create a new transaction 
        pub fn new<R>(csprng: &mut R, dest_pk: &PublicKey, transfer_amount: u32, account_balance: u32, account_blind: Scalar, receiver_commit: RistrettoPoint) -> (Transaction, Scalar) 
                where R: CryptoRng + Rng, 
        {
                //public params
                let mut params = PublicParams::new();
                //1. Sample Fresh blinding factor [blind], its a scalar
                let blinding_t = Scalar::random(csprng);

                //2. Create Commitment ->  g^amount * h^[blind] == CommT
                //let commit_t = pc_gens.commit(Scalar::from(transfer_amount), blinding_t);

                //4. create Commitment ->  g^(Balance - amount) * h^(Opening - blind) == CommS
                let sender_updated_balance = account_balance - transfer_amount;

                //3. Create rangeproof for amount & use [blind] as randomness == RP_T
                //5. Create rangeproof for (Balance - transfer_amount) & use Opening - blind as randomness == RP_S
                //updated account blind
                let sender_updated_account_blind = account_blind - blinding_t;

                // Create an aggregated 32-bit rangeproof and corresponding commitments.
                let (proof_agg, commitments_agg) = RangeProof::prove_multiple(
                        &params.bp_gens,
                        &params.pc_gens,
                        &mut params.transcript,
                        &[u64::from(transfer_amount), u64::from(sender_updated_balance)],
                        &[blinding_t, sender_updated_account_blind],
                        32,
                ).expect("HANDLE ERRORS BETTER");


                //6. Multiply Commitment ->  oldCommR * CommT == CommR
                let new_commit_receiver = receiver_commit + commitments_agg[0].decompress().unwrap();

                //7. Encrypt to receiver pubkey both the transfer_amount transferred and the blinding factor [blind] 
                let mut to_encrypt = Vec::new();
                //first add transfer_amount which is fixed 4 bytes in big endian
                to_encrypt.extend_from_slice(&be_u8_from_u32(transfer_amount));
                //next add the blind
                to_encrypt.extend_from_slice(&blinding_t.to_bytes());
                //lock em up
                let lbox = Lockbox::lock(csprng, dest_pk, &to_encrypt);

                //return transaction structure and new blind
                (Transaction {
                        transaction_range_proof: proof_agg,
                        transaction_commitment: commitments_agg[0],
                        sender_updated_balance_commitment: commitments_agg[1],
                        receiver_new_commit: new_commit_receiver.compress(),
                        lockbox: lbox
                }, sender_updated_account_blind)
        }

        //helper function to recover the sent amount and blind factor
        pub fn recover_plaintext(&self, sk: &SecretKey) -> (u32, Scalar) {
                //unlock encrypted box
                let unlocked = self.lockbox.unlock(sk);
                //extract balance value & blind value
                let (raw_amount, raw_blind) = unlocked.split_at(5);

                //convert to u32
                let p_amount = u32::from(raw_amount[0]) << 24 |
                u32::from(raw_amount[1]) << 16 |
                u32::from(raw_amount[2]) << 8 |
                u32::from(raw_amount[3]);

                //recover blind from bytes to scalar
                let recovered_blind_scalar = Scalar::from_bits(slice_to_fixed32(raw_blind));

                (p_amount, recovered_blind_scalar)
        }

}


//verify transaction used by validator.
//We just check if the public visible parts are correctly computed 
pub fn validator_verify(tx: &Transaction, sender_prev_com: RistrettoPoint, receiver_prev_com: RistrettoPoint) -> bool {
        //Common Reference String
        let mut transcript = Transcript::new(b"Zei Range Proof");
        //def pederson from lib with Common Reference String
        let pc_gens = PedersenGens::default();
        //32bit range for now & one prover
        let bp_gens = BulletproofGens::new(32, 2);
     
        //We start our verification pipeline with the commitment calculations as cheaper than rangeproof.

        //1. the sender commitment is old from network - this tx commitment
        let derived_sender_com = sender_prev_com - tx.transaction_commitment.decompress().unwrap();
        if derived_sender_com == tx.sender_updated_balance_commitment.decompress().unwrap() {
                //2. the receiver commitment is old from network + this tx commitment
                let derived_receiver_com = receiver_prev_com + tx.transaction_commitment.decompress().unwrap();
                if derived_receiver_com == tx.receiver_new_commit.decompress().unwrap() {

                        //verify the sender proofs
                        let verify_t = RangeProof::verify_multiple(
                                &tx.transaction_range_proof,
                                &bp_gens,
                                &pc_gens,
                                &mut transcript,
                                &[tx.transaction_commitment, tx.sender_updated_balance_commitment],
                                32
                        );

                        //check rangeproof
                        verify_t.is_ok()
                } else { 
                        false
                }
        } else { 
                false
        }

}


// verify commitments
pub fn receiver_verify(tx_amount: u32, tx_blind: Scalar, new_commit: RistrettoPoint, recv_old_commit: RistrettoPoint) -> bool {
        // def pederson from lib with Common Reference String
        use bulletproofs::PedersenGens;
        let pc_gens = PedersenGens::default();

        let compute_new_commit = pc_gens.commit(Scalar::from(tx_amount), tx_blind);

        let updated_commitment = compute_new_commit + recv_old_commit;

        new_commit == updated_commitment
}




#[cfg(test)]
mod test {
        use super::*;
        use crate::account::Account;
        use curve25519_dalek::scalar::Scalar;
        use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
        use merlin::Transcript;
        use rand::ChaChaRng;
        use rand::SeedableRng;

        #[test]
        fn test_new_transaction() {
                let mut csprng: ChaChaRng;
                csprng  = ChaChaRng::from_seed([0u8; 32]);

                //def pederson from lib with Common Reference String
                let pc_gens = PedersenGens::default();

                //Account A
                let mut acc_a = Account::new(&mut csprng);
                //Account B
                let mut acc_b = Account::new(&mut csprng);

                //the initial commitment is to zero
                let acc_a_comm_inital = pc_gens.commit(Scalar::from(0u32), Scalar::from(0u32));
                let acc_b_comm_inital = pc_gens.commit(Scalar::from(0u32), Scalar::from(0u32));

                let new_tx = CreateTx {
                        receiver: acc_b.keys.public,
                        receiver_commit: acc_b.commitment,
                        transfer_amount: 100u32
                };
                
                //
                //Create Proofs
                //

                let mut transcript = Transcript::new(b"Zei Range Proof");
                //32bit range for now & one prover
                let bp_gens = BulletproofGens::new(32, 2);

                //1. Sample Fresh blinding factor [blind], its a scalar
                // let mut csprng: OsRng = OsRng::new().unwrap();
                let blinding_t = Scalar::random(&mut csprng);

                //update sending account balance 
                //acc_a.balance = acc_a.balance - new_tx.transfer_amount; 
                //update account blind 
                let sender_updated_account_blind = &acc_a.opening - &blinding_t;
                
                // Create an aggregated 32-bit rangeproof and corresponding commitments.
                let (proof_agg, commitments_agg) = RangeProof::prove_multiple(
                        &bp_gens,
                        &pc_gens,
                        &mut transcript,
                        &[new_tx.transfer_amount as u64, acc_a.balance as u64],
                        &[blinding_t, acc_a.opening],
                        32,
                ).expect("A real program could handle errors");

                let tx_derived_commit = pc_gens.commit(Scalar::from(new_tx.transfer_amount), blinding_t);
                //println!("tx_derived_commit: {:?}", tx_derived_commit.compress());
                //println!("commitments_agg[0]: {:?}", commitments_agg[0]);

                assert_eq!(tx_derived_commit, commitments_agg[0].decompress().unwrap());
                //create a dummy tx
                //let tx = new_transaction(new_tx.receiver, new_tx.transfer_amount, acc_a.balance, acc_a.commitment, new_tx.receiver_commit);

                //verify reciver commitment
                //assert_eq!(receiver_verify(p_amount, recovered_blind_scalar, tx.receiver_new_commit, acc_b_comm_inital), true);
                // pub fn new_transaction(dest_pk: &PublicKey, transfer_amount: u32, account_balance: u32, account_blind: Scalar, receiver_commit: RistrettoPoint) -> Transaction {
                
                //7. Encrypt to receiver pubkey both the transfer_amount transferred and the blinding factor [blind] 
                // let mut to_encrypt = Vec::new();
                // //first add transfer_amount which is fixed 4 bytes in big endian
                // to_encrypt.extend_from_slice(&be_u8_from_u32(transfer_amount));
                // //next add the blind
                // to_encrypt.extend_from_slice(&blinding_t.to_bytes());
                // //lock em up
                // let lbox = Lockbox::lock(dest_pk, &to_encrypt);

                // let final_tx = Transaction {
                //         transaction_range_proof: range_proof_t,
                //         transaction_commitment: commit_t,
                //         sender_updated_balance_range_proof: range_proof_s,
                //         sender_updated_balance_commitment: commit_s,
                //         receiver_new_commit: new_commit_receiver.compress(),
                //         lockbox: lbox
                // };
        }
}

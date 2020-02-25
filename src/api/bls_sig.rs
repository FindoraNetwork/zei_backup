// BLS Signatures
use crate::algebra::bls12_381::{BLSScalar, Bls12381, BLSG1};
use crate::basic_crypto::signatures::{AggSignatureTrait, SignatureTrait};
use crate::errors::ZeiError;
use rand_core::{CryptoRng, RngCore};

pub type BlsSecretKey = crate::basic_crypto::signatures::bls::BlsSecretKey<Bls12381>;
pub type BlsPublicKey = crate::basic_crypto::signatures::bls::BlsPublicKey<Bls12381>;
pub type BlsSignature = crate::basic_crypto::signatures::bls::BlsSignature<Bls12381>;

/// bls key generation function
pub fn bls_gen_keys<R: CryptoRng + RngCore>(prng: &mut R) -> (BlsSecretKey, BlsPublicKey) {
  crate::basic_crypto::signatures::bls::bls_gen_keys::<_, Bls12381>(prng)
}

/// bls signature function
pub fn bls_sign<B: AsRef<[u8]>>(signing_key: &BlsSecretKey, message: &B) -> BlsSignature {
  crate::basic_crypto::signatures::bls::bls_sign::<Bls12381, B>(signing_key, message)
}

/// bls verification function
pub fn bls_verify<B: AsRef<[u8]>>(ver_key: &BlsPublicKey,
                                  message: &B,
                                  signature: &BlsSignature)
                                  -> Result<(), ZeiError> {
  crate::basic_crypto::signatures::bls::bls_verify::<Bls12381, B>(ver_key, message, signature)
}

/// aggregate signature (for a single common message)
pub fn bls_aggregate(ver_keys: &[&BlsPublicKey], signatures: &[&BlsSignature]) -> BlsSignature {
  crate::basic_crypto::signatures::bls::bls_aggregate::<Bls12381>(ver_keys, signatures)
}

/// Verification of an aggregated signature for a common message
pub fn bls_verify_aggregated<B: AsRef<[u8]>>(ver_keys: &[&BlsPublicKey],
                                             message: &B,
                                             agg_signature: &BlsSignature)
                                             -> Result<(), ZeiError> {
  crate::basic_crypto::signatures::bls::bls_verify_aggregated::<Bls12381, B>(ver_keys,
                                                                             message,
                                                                             agg_signature)
}

/// Batch verification of many signatures
pub fn bls_batch_verify<B: AsRef<[u8]>>(ver_keys: &[BlsPublicKey],
                                        messages: &[B],
                                        signatures: &[BlsSignature])
                                        -> Result<(), ZeiError> {
  crate::basic_crypto::signatures::bls::bls_batch_verify::<Bls12381, B>(ver_keys, messages,
                                                                        signatures)
}

/// signature aggregation for (possibly) different messages
pub fn bls_add_signatures(signatures: &[BlsSignature]) -> BlsSignature {
  crate::basic_crypto::signatures::bls::bls_add_signatures::<Bls12381>(signatures)
}

/// verification of an aggregated signatures for different messages
pub fn bls_batch_verify_added_signatures<B: AsRef<[u8]>>(ver_keys: &[BlsPublicKey],
                                                         messages: &[B],
                                                         signature: &BlsSignature)
                                                         -> Result<(), ZeiError> {
  crate::basic_crypto::signatures::bls::bls_batch_verify_added_signatures::<Bls12381, B>(ver_keys,
                                                                                         messages,
                                                                                         signature)
}

pub struct Bls;

impl SignatureTrait for Bls {
  type PublicKey = BlsPublicKey;
  type SecretKey = BlsSecretKey;
  type Signature = BlsSignature;
  fn gen_keys<R: CryptoRng + RngCore>(prng: &mut R) -> (BlsSecretKey, BlsPublicKey) {
    bls_gen_keys(prng)
  }
  fn sign<B: AsRef<[u8]>>(sk: &Self::SecretKey, msg: &B) -> Self::Signature {
    bls_sign(sk, msg)
  }
  fn verify<B: AsRef<[u8]>>(pk: &Self::PublicKey,
                            sig: &Self::Signature,
                            msg: &B)
                            -> Result<(), ZeiError> {
    bls_verify(pk, msg, sig)
  }
}

impl AggSignatureTrait for Bls {
  type AggSignature = BlsSignature;
  fn aggregate<B: AsRef<[u8]>>(pks: &[&Self::PublicKey],
                               sigs: &[&Self::Signature])
                               -> Self::AggSignature {
    bls_aggregate(pks, sigs)
  }
  fn verify_aggregate<B: AsRef<[u8]>>(pks: &[&Self::PublicKey],
                                      agg_sig: &Self::AggSignature,
                                      msg: &B)
                                      -> Result<(), ZeiError> {
    bls_verify_aggregated(pks, msg, agg_sig)
  }
}

impl Into<BLSG1> for BlsPublicKey {
  fn into(self) -> BLSG1 {
    self.0
  }
}

impl AsRef<BLSG1> for BlsPublicKey {
  fn as_ref(&self) -> &BLSG1 {
    &self.0
  }
}

impl Into<BLSScalar> for BlsSecretKey {
  fn into(self) -> BLSScalar {
    self.0
  }
}

impl AsRef<BLSScalar> for BlsSecretKey {
  fn as_ref(&self) -> &BLSScalar {
    &self.0
  }
}

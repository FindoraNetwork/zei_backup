use super::groups::{Group, Scalar};
use super::pairing::Pairing;
use crate::algebra::groups::GroupArithmetic;
use crate::utils::{b64dec, b64enc, u8_bigendian_slice_to_u64};
use crate::utils::{u64_to_bigendian_u8array, u8_bigendian_slice_to_u32};
use digest::generic_array::typenum::U64;
use digest::Digest;
use pairing::bls12_381::{Fq, Fq12, Fq2, Fq6, FqRepr, Fr, FrRepr, G1, G2};
use pairing::{CurveAffine, CurveProjective};
use pairing::{EncodedPoint, Field, PrimeField};
use rand_04::Rand;
use rand_core::{CryptoRng, RngCore};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BLSScalar(pub(crate) Fr);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BLSG1(pub(crate) G1);
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BLSG2(pub(crate) G2);
#[derive(Clone, PartialEq, Eq)]
pub struct BLSGt(pub(crate) Fq12);

impl Scalar for BLSScalar {
  // scalar generation
  fn random_scalar<R: CryptoRng + RngCore>(rng: &mut R) -> BLSScalar {
    // hack to use rand_04::Rng rather than rand::Rng
    let mut random_bytes = [0u8; 16];
    rng.fill_bytes(&mut random_bytes);
    let mut seed = [0u32; 4];
    for i in 0..4 {
      seed[i] = u8_bigendian_slice_to_u32(&random_bytes[i * 4..(i + 1) * 4]);
    }

    use rand_04::SeedableRng;
    let mut prng_04 = rand_04::ChaChaRng::from_seed(&seed);
    BLSScalar(Fr::rand(&mut prng_04))
  }

  fn from_u32(value: u32) -> BLSScalar {
    Self::from_u64(value as u64)
  }

  fn from_u64(value: u64) -> BLSScalar {
    let mut v = value;
    let mut result = Fr::zero();
    let mut two_pow_i = Fr::one();
    for _ in 0..64 {
      if v == 0 {
        break;
      }
      if v & 1 == 1u64 {
        result.add_assign(&two_pow_i);
        //result = result + two_pow_i;
      }
      v >>= 1;
      two_pow_i.double(); // = two_pow_i * two;
    }
    BLSScalar(result)
  }

  fn from_hash<D>(hash: D) -> BLSScalar
    where D: Digest<OutputSize = U64> + Default
  {
    let result = hash.result();
    let mut seed = [0u32; 16];
    for (i, item) in seed.iter_mut().enumerate() {
      *item = u8_bigendian_slice_to_u32(&result.as_slice()[i * 4..(i + 1) * 4]);
    }
    use rand_04::SeedableRng;
    let mut prng = rand_04::ChaChaRng::from_seed(&seed);
    BLSScalar(Fr::rand(&mut prng))
  }

  // scalar arithmetic
  fn add(&self, b: &BLSScalar) -> BLSScalar {
    let mut m = self.0;
    m.add_assign(&b.0);
    BLSScalar(m)
  }
  fn mul(&self, b: &BLSScalar) -> BLSScalar {
    let mut m = self.0;
    m.mul_assign(&b.0);
    BLSScalar(m)
  }

  fn sub(&self, b: &BLSScalar) -> BLSScalar {
    let mut m = self.0;
    m.sub_assign(&b.0);
    BLSScalar(m)
  }

  fn inv(&self) -> BLSScalar {
    BLSScalar((self.0).inverse().unwrap())
  }

  fn get_little_endian_u64(&self) -> Vec<u64> {
    (self.0).into_repr().0.to_vec()
  }
  //scalar serialization
  fn to_bytes(&self) -> Vec<u8> {
    let repr = FrRepr::from(self.0);
    let mut v = vec![];
    for a in &repr.0 {
      let array = crate::utils::u64_to_bigendian_u8array(*a);
      v.extend_from_slice(&array[..])
    }
    v
  }

  fn from_bytes(bytes: &[u8]) -> BLSScalar {
    let mut repr_array = [0u64; 4];
    for i in 0..4 {
      let slice = &bytes[i * 8..i * 8 + 8];
      repr_array[i] = crate::utils::u8_bigendian_slice_to_u64(slice);
    }
    let fr_repr = FrRepr(repr_array);
    BLSScalar(Fr::from_repr(fr_repr).unwrap())
  }
}

impl Serialize for BLSScalar {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
  {
    if serializer.is_human_readable() {
      serializer.serialize_str(&b64enc(self.to_bytes().as_slice()))
    } else {
      serializer.serialize_bytes(self.to_bytes().as_slice())
    }
  }
}

impl<'de> Deserialize<'de> for BLSScalar {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
  {
    struct ScalarVisitor;

    impl<'de> Visitor<'de> for ScalarVisitor {
      type Value = BLSScalar;

      fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        formatter.write_str("a encoded BLSG2 element")
      }

      fn visit_bytes<E>(self, v: &[u8]) -> Result<BLSScalar, E>
        where E: serde::de::Error
      {
        Ok(BLSScalar::from_bytes(v))
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<BLSScalar, V::Error>
        where V: SeqAccess<'de>
      {
        let mut vec: Vec<u8> = vec![];
        while let Some(x) = seq.next_element().unwrap() {
          vec.push(x);
        }
        Ok(BLSScalar::from_bytes(vec.as_slice()))
      }
      fn visit_str<E>(self, s: &str) -> Result<BLSScalar, E>
        where E: serde::de::Error
      {
        self.visit_bytes(&b64dec(s).map_err(serde::de::Error::custom)?)
      }
    }
    if deserializer.is_human_readable() {
      deserializer.deserialize_str(ScalarVisitor)
    } else {
      deserializer.deserialize_bytes(ScalarVisitor)
    }
  }
}

impl Group<BLSScalar> for BLSG1 {
  const COMPRESSED_LEN: usize = 48;
  const SCALAR_BYTES_LEN: usize = 32;
  fn get_identity() -> BLSG1 {
    BLSG1(G1::zero())
  }
  fn get_base() -> BLSG1 {
    BLSG1(G1::one())
  }

  // compression/serialization helpers
  fn to_compressed_bytes(&self) -> Vec<u8> {
    let v = self.0.into_affine().into_compressed().as_ref().to_vec();
    v
  }
  fn from_compressed_bytes(bytes: &[u8]) -> Option<BLSG1> {
    let some: G1 = G1::one();
    let mut compressed = some.into_affine().into_compressed();
    let mut_bytes = compressed.as_mut();
    mut_bytes[..48].clone_from_slice(&bytes[..48]);
    let affine = compressed.into_affine().unwrap();
    let g1 = G1::from(affine);

    Some(BLSG1(g1))
  }

  fn from_hash<D>(hash: D) -> BLSG1
    where D: Digest<OutputSize = U64> + Default
  {
    let result = hash.result();
    let mut seed = [0u32; 16];
    for (i, item) in seed.iter_mut().enumerate() {
      *item = u8_bigendian_slice_to_u32(&result.as_slice()[i * 4..(i + 1) * 4]);
    }
    use rand_04::SeedableRng;
    let mut prng = rand_04::ChaChaRng::from_seed(&seed);
    BLSG1(G1::rand(&mut prng))
  }
}

impl GroupArithmetic<BLSScalar> for BLSG1 {
  //arithmetic
  fn mul(&self, scalar: &BLSScalar) -> BLSG1 {
    let mut m = self.0;
    m.mul_assign(scalar.0);
    BLSG1(m)
  }
  fn add(&self, other: &Self) -> BLSG1 {
    let mut m = self.0;
    m.add_assign(&other.0);
    BLSG1(m)
  }
  fn sub(&self, other: &Self) -> BLSG1 {
    let mut m = self.0;
    m.sub_assign(&other.0);
    BLSG1(m)
  }
}

impl Serialize for BLSG1 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
  {
    if serializer.is_human_readable() {
      serializer.serialize_str(&b64enc(self.to_compressed_bytes().as_slice()))
    } else {
      serializer.serialize_bytes(self.to_compressed_bytes().as_slice())
    }
  }
}

impl<'de> Deserialize<'de> for BLSG1 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
  {
    struct G1Visitor;

    impl<'de> Visitor<'de> for G1Visitor {
      type Value = BLSG1;

      fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        formatter.write_str("a encoded ElGamal Ciphertext")
      }

      fn visit_bytes<E>(self, v: &[u8]) -> Result<BLSG1, E>
        where E: serde::de::Error
      {
        Ok(BLSG1::from_compressed_bytes(v).unwrap()) //TODO handle error
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<BLSG1, V::Error>
        where V: SeqAccess<'de>
      {
        let mut vec: Vec<u8> = vec![];
        while let Some(x) = seq.next_element().unwrap() {
          vec.push(x);
        }
        Ok(BLSG1::from_compressed_bytes(vec.as_slice()).unwrap())
      }
      fn visit_str<E>(self, s: &str) -> Result<BLSG1, E>
        where E: serde::de::Error
      {
        self.visit_bytes(&b64dec(s).map_err(serde::de::Error::custom)?)
      }
    }
    if deserializer.is_human_readable() {
      deserializer.deserialize_str(G1Visitor)
    } else {
      deserializer.deserialize_bytes(G1Visitor)
    }
  }
}

impl Group<BLSScalar> for BLSG2 {
  const COMPRESSED_LEN: usize = 96; // TODO
  const SCALAR_BYTES_LEN: usize = 32; // TODO
  fn get_identity() -> BLSG2 {
    BLSG2(G2::zero())
  }
  fn get_base() -> BLSG2 {
    BLSG2(G2::one())
  }

  // compression/serialization helpers
  fn to_compressed_bytes(&self) -> Vec<u8> {
    let v = self.0.into_affine().into_compressed().as_ref().to_vec();
    v
  }
  #[allow(clippy::manual_memcpy)]
  fn from_compressed_bytes(bytes: &[u8]) -> Option<BLSG2> {
    let some: G2 = G2::one();
    let mut compressed = some.into_affine().into_compressed();
    let mut_bytes = compressed.as_mut();
    for i in 0..96 {
      mut_bytes[i] = bytes[i];
    }
    let affine = compressed.into_affine().unwrap();
    let g2 = G2::from(affine);

    Some(BLSG2(g2))
  }

  fn from_hash<D>(hash: D) -> BLSG2
    where D: Digest<OutputSize = U64> + Default
  {
    let result = hash.result();
    let mut seed = [0u32; 16];
    for (i, item) in seed.iter_mut().enumerate() {
      *item = u8_bigendian_slice_to_u32(&result.as_slice()[i * 4..(i + 1) * 4]);
    }
    use rand_04::SeedableRng;
    let mut prng = rand_04::ChaChaRng::from_seed(&seed);
    BLSG2(G2::rand(&mut prng))
  }
}

impl GroupArithmetic<BLSScalar> for BLSG2 {
  //arithmetic
  fn mul(&self, scalar: &BLSScalar) -> BLSG2 {
    let mut m = self.0;
    m.mul_assign(scalar.0);
    BLSG2(m)
  }
  fn add(&self, other: &Self) -> BLSG2 {
    let mut m = self.0;
    m.add_assign(&other.0);
    BLSG2(m)
  }
  fn sub(&self, other: &Self) -> BLSG2 {
    let mut m = self.0;
    m.sub_assign(&other.0);
    BLSG2(m)
  }
}

impl Serialize for BLSG2 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
  {
    if serializer.is_human_readable() {
      serializer.serialize_str(&b64enc(self.to_compressed_bytes().as_slice()))
    } else {
      serializer.serialize_bytes(self.to_compressed_bytes().as_slice())
    }
  }
}

impl<'de> Deserialize<'de> for BLSG2 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
  {
    struct G2Visitor;

    impl<'de> Visitor<'de> for G2Visitor {
      type Value = BLSG2;

      fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        formatter.write_str("a encoded BLSG2 element")
      }

      fn visit_bytes<E>(self, v: &[u8]) -> Result<BLSG2, E>
        where E: serde::de::Error
      {
        Ok(BLSG2::from_compressed_bytes(v).unwrap()) //TODO handle error
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<BLSG2, V::Error>
        where V: SeqAccess<'de>
      {
        let mut vec: Vec<u8> = vec![];
        while let Some(x) = seq.next_element().unwrap() {
          vec.push(x);
        }
        Ok(BLSG2::from_compressed_bytes(vec.as_slice()).unwrap())
      }
      fn visit_str<E>(self, s: &str) -> Result<BLSG2, E>
        where E: serde::de::Error
      {
        self.visit_bytes(&b64dec(s).map_err(serde::de::Error::custom)?)
      }
    }
    if deserializer.is_human_readable() {
      deserializer.deserialize_str(G2Visitor)
    } else {
      deserializer.deserialize_bytes(G2Visitor)
    }
  }
}

impl fmt::Debug for BLSGt {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Fr: Some Gt Element")
  }
}

pub struct Bls12381;

fn bls_pairing(a: &BLSG1, b: &BLSG2) -> BLSGt {
  BLSGt(a.0.into_affine().pairing_with(&b.0.into_affine()))
}

impl Pairing for Bls12381 {
  type ScalarField = BLSScalar;
  type G1 = BLSG1;
  type G2 = BLSG2;
  type Gt = BLSGt;

  fn pairing(a: &Self::G1, b: &Self::G2) -> Self::Gt {
    bls_pairing(a, b)
  }
}

impl GroupArithmetic<BLSScalar> for BLSGt {
  fn mul(&self, scalar: &BLSScalar) -> Self {
    let r = self.0.pow(scalar.0.into_repr().as_ref());
    BLSGt(r)
  }
  fn add(&self, other: &Self) -> Self {
    let mut m = other.0;
    m.mul_assign(&self.0);
    BLSGt(m)
  }
  fn sub(&self, other: &Self) -> Self {
    let mut result = other.0.inverse().unwrap();
    result.mul_assign(&self.0);
    BLSGt(result)
  }
}

impl Group<BLSScalar> for BLSGt {
  const COMPRESSED_LEN: usize = 576;
  const SCALAR_BYTES_LEN: usize = 32; // TODO
  fn get_identity() -> BLSGt {
    BLSGt(Fq12::one())
  }

  fn get_base() -> Self {
    bls_pairing(&BLSG1::get_base(), &BLSG2::get_base()) //TODO hardcode this
  }

  // compression/serialization helpers
  fn to_compressed_bytes(&self) -> Vec<u8> {
    let c0c0c0 = self.0.c0.c0.c0.into_repr();
    let c0c0c1 = self.0.c0.c0.c1.into_repr();
    let c0c1c0 = self.0.c0.c1.c0.into_repr();
    let c0c1c1 = self.0.c0.c1.c1.into_repr();
    let c0c2c0 = self.0.c0.c2.c0.into_repr();
    let c0c2c1 = self.0.c0.c2.c1.into_repr();

    let c1c0c0 = self.0.c1.c0.c0.into_repr();
    let c1c0c1 = self.0.c1.c0.c1.into_repr();
    let c1c1c0 = self.0.c1.c1.c0.into_repr();
    let c1c1c1 = self.0.c1.c1.c1.into_repr();
    let c1c2c0 = self.0.c1.c2.c0.into_repr();
    let c1c2c1 = self.0.c1.c2.c1.into_repr();

    let mut v = vec![];
    v.extend_from_slice(&c0c0c0.0[..]);
    v.extend_from_slice(&c0c0c1.0[..]);
    v.extend_from_slice(&c0c1c0.0[..]);
    v.extend_from_slice(&c0c1c1.0[..]);
    v.extend_from_slice(&c0c2c0.0[..]);
    v.extend_from_slice(&c0c2c1.0[..]);

    v.extend_from_slice(&c1c0c0.0[..]);
    v.extend_from_slice(&c1c0c1.0[..]);
    v.extend_from_slice(&c1c1c0.0[..]);
    v.extend_from_slice(&c1c1c1.0[..]);
    v.extend_from_slice(&c1c2c0.0[..]);
    v.extend_from_slice(&c1c2c1.0[..]);

    let mut r = vec![];
    for vi in v {
      r.extend_from_slice(&u64_to_bigendian_u8array(vi)[..]);
    }
    r
  }

  fn from_compressed_bytes(v: &[u8]) -> Option<Self> {
    Some(BLSGt(Fq12 { c0: build_fq6(&v[..v.len() / 2])?,
                      c1: build_fq6(&v[v.len() / 2..])? }))
  }

  fn from_hash<D>(hash: D) -> Self
    where D: Digest<OutputSize = U64> + Default
  {
    let result = hash.result();
    let mut seed = [0u32; 16];
    for (i, item) in seed.iter_mut().enumerate() {
      *item = u8_bigendian_slice_to_u32(&result.as_slice()[i * 4..(i + 1) * 4]);
    }
    use rand_04::SeedableRng;
    let mut prng = rand_04::ChaChaRng::from_seed(&seed);
    BLSGt(Fq12::rand(&mut prng))
  }
}

fn build_fq6(v: &[u8]) -> Option<Fq6> {
  let n = v.len() / 3;
  Some(Fq6 { c0: build_fq2(&v[..n])?,
             c1: build_fq2(&v[n..2 * n])?,
             c2: build_fq2(&v[2 * n..])? })
}

fn build_fq2(v: &[u8]) -> Option<Fq2> {
  let n = v.len() / 2;
  Some(Fq2 { c0: build_fq(&v[..n])?,
             c1: build_fq(&v[n..])? })
}

fn build_fq(v: &[u8]) -> Option<Fq> {
  if v.len() != 48 {
    return None;
  }
  let a1 = u8_bigendian_slice_to_u64(&v[0..8]);
  let a2 = u8_bigendian_slice_to_u64(&v[8..16]);
  let a3 = u8_bigendian_slice_to_u64(&v[16..24]);
  let a4 = u8_bigendian_slice_to_u64(&v[24..32]);
  let a5 = u8_bigendian_slice_to_u64(&v[32..40]);
  let a6 = u8_bigendian_slice_to_u64(&v[40..]);
  Some(Fq::from_repr(FqRepr([a1, a2, a3, a4, a5, a6])).ok()?)
}
impl Serialize for BLSGt {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
  {
    if serializer.is_human_readable() {
      serializer.serialize_str(&b64enc(self.to_compressed_bytes().as_slice()))
    } else {
      serializer.serialize_bytes(self.to_compressed_bytes().as_slice())
    }
  }
}

impl<'de> Deserialize<'de> for BLSGt {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
  {
    struct GtVisitor;

    impl<'de> Visitor<'de> for GtVisitor {
      type Value = BLSGt;

      fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        formatter.write_str("a encoded BLSGt element")
      }

      fn visit_bytes<E>(self, v: &[u8]) -> Result<BLSGt, E>
        where E: serde::de::Error
      {
        Ok(BLSGt::from_compressed_bytes(v).unwrap()) //TODO handle error
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<BLSGt, V::Error>
        where V: SeqAccess<'de>
      {
        let mut vec: Vec<u8> = vec![];
        while let Some(x) = seq.next_element().unwrap() {
          vec.push(x);
        }
        Ok(BLSGt::from_compressed_bytes(vec.as_slice()).unwrap())
      }
      fn visit_str<E>(self, s: &str) -> Result<BLSGt, E>
        where E: serde::de::Error
      {
        self.visit_bytes(&b64dec(s).map_err(serde::de::Error::custom)?)
      }
    }
    if deserializer.is_human_readable() {
      deserializer.deserialize_str(GtVisitor)
    } else {
      deserializer.deserialize_bytes(GtVisitor)
    }
  }
}

#[cfg(test)]
mod bls12_381_groups_test {
  use crate::algebra::groups::group_tests::{test_scalar_operations, test_scalar_serialization};

  #[test]
  fn test_scalar_ops() {
    test_scalar_operations::<super::BLSScalar>();
  }

  #[test]
  fn scalar_deser() {
    test_scalar_serialization::<super::BLSScalar>();
  }
}

#[cfg(test)]
mod elgamal_over_bls_groups {
  use crate::basic_crypto::elgamal::elgamal_test;

  #[test]
  fn verification_g1() {
    elgamal_test::verification::<super::BLSScalar, super::BLSG1>();
  }

  #[test]
  fn decryption_g1() {
    elgamal_test::decryption::<super::BLSScalar, super::BLSG1>();
  }

  #[test]
  fn to_json_g1() {
    elgamal_test::to_json::<super::BLSScalar, super::BLSG1>();
  }

  #[test]
  fn to_message_pack_g1() {
    elgamal_test::to_message_pack::<super::BLSScalar, super::BLSG1>();
  }

  #[test]
  fn verification_g2() {
    elgamal_test::verification::<super::BLSScalar, super::BLSG2>();
  }

  #[test]
  fn decryption_g2() {
    elgamal_test::decryption::<super::BLSScalar, super::BLSG2>();
  }

  #[test]
  fn to_json_g2() {
    elgamal_test::to_json::<super::BLSScalar, super::BLSG2>();
  }

  #[test]
  fn to_message_pack_g2() {
    elgamal_test::to_message_pack::<super::BLSScalar, super::BLSG2>();
  }

  #[test]
  fn verification_gt() {
    elgamal_test::verification::<super::BLSScalar, super::BLSGt>();
  }

  #[test]
  fn decryption_gt() {
    elgamal_test::decryption::<super::BLSScalar, super::BLSGt>();
  }

  #[test]
  fn to_json_gt() {
    elgamal_test::to_json::<super::BLSScalar, super::BLSGt>();
  }

  #[test]
  fn to_message_pack_gt() {
    elgamal_test::to_message_pack::<super::BLSScalar, super::BLSGt>();
  }
}

#[cfg(test)]
mod credentials_over_bls_12_381 {

  use crate::crypto::anon_creds::credentials_tests;

  #[test]
  fn single_attribute() {
    credentials_tests::single_attribute::<super::Bls12381>();
  }

  #[test]
  fn two_attributes() {
    credentials_tests::two_attributes::<super::Bls12381>();
  }

  #[test]
  fn ten_attributes() {
    credentials_tests::ten_attributes::<super::Bls12381>();
  }

  #[test]
  fn to_json_credential_structures() {
    credentials_tests::to_json_credential_structures::<super::Bls12381>();
  }

  #[test]
  fn to_msg_pack_credential_structures() {
    credentials_tests::to_msg_pack_credential_structures::<super::Bls12381>();
  }

  /*
  #[test]
  fn to_json_issuer_priv_key(){
      crate::credentials::credentials_tests::to_json_issuer_priv_key::<super::BLSGt>();
  }

  #[test]
  fn to_msg_pack_issuer_priv_key(){
      crate::credentials::credentials_tests::to_msg_pack_issuer_priv_key::<super::BLSGt>();
  }

  #[test]
  fn to_json_user_pub_key(){
      crate::credentials::credentials_tests::to_json_user_pub_key::<super::BLSGt>();
  }

  #[test]
  fn to_msg_pack_user_pub_key(){
      crate::credentials::credentials_tests::to_msg_pack_user_pub_key::<super::BLSGt>();
  }
  */
}

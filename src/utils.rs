// Utility functions

use crate::algebra::groups::Scalar;
use digest::Digest;
use sha2::Sha512;

/// I convert a u32 into a 4 bytes array (bigendian)
pub(crate) fn u32_to_bigendian_u8array(n: u32) -> [u8; 4] {
  let mut array = [0u8; 4];
  array[0] = ((n >> 24) & 0xFF) as u8;
  array[1] = ((n >> 16) & 0xFF) as u8;
  array[2] = ((n >> 8) & 0xFF) as u8;
  array[3] = (n & 0xFF) as u8;
  array
}

/// I convert a u64 into a 8 bytes array (bigendian)
pub(crate) fn u64_to_bigendian_u8array(n: u64) -> [u8; 8] {
  let mut array = [0u8; 8];
  array[0] = ((n >> 56) & 0xFF) as u8;
  array[1] = ((n >> 48) & 0xFF) as u8;
  array[2] = ((n >> 40) & 0xFF) as u8;
  array[3] = ((n >> 32) & 0xFF) as u8;
  array[4] = ((n >> 24) & 0xFF) as u8;
  array[5] = ((n >> 16) & 0xFF) as u8;
  array[6] = ((n >> 8) & 0xFF) as u8;
  array[7] = (n & 0xFF) as u8;
  array
}

/*
pub fn u128_to_bigendian_u8array(n: u128) -> [u8;16]{
    let mut array = [0u8;16];
    array[0] = ((n>>120)&0xFF) as u8;
    array[1] = ((n>>112)&0xFF) as u8;
    array[2] = ((n>>104)&0xFF) as u8;
    array[3] = ((n>>96)&0xFF) as u8;
    array[4] = ((n>>88)&0xFF) as u8;
    array[5] = ((n>>80)&0xFF) as u8;
    array[6] = ((n>>72)&0xFF) as u8;
    array[7] = ((n>>64)&0xFF) as u8;
    array[8] = ((n>>56)&0xFF) as u8;
    array[9] = ((n>>48)&0xFF) as u8;
    array[10] = ((n>>40)&0xFF) as u8;
    array[11] = ((n>>32)&0xFF) as u8;
    array[12] = ((n>>24)&0xFF) as u8;
    array[13] = ((n>>16)&0xFF) as u8;
    array[14] = ((n>>8)&0xFF) as u8;
    array[15] = (n & 0xFF) as u8;
    array
}
*/

/// I convert a 16 byte array into a u128 (bigendian)
pub(crate) fn u8_bigendian_slice_to_u128(array: &[u8]) -> u128 {
  u128::from(array[0]) << 120
  | u128::from(array[1]) << 112
  | u128::from(array[2]) << 104
  | u128::from(array[3]) << 96
  | u128::from(array[4]) << 88
  | u128::from(array[5]) << 80
  | u128::from(array[6]) << 72
  | u128::from(array[7]) << 64
  | u128::from(array[8]) << 56
  | u128::from(array[9]) << 48
  | u128::from(array[10]) << 40
  | u128::from(array[11]) << 32
  | u128::from(array[12]) << 24
  | u128::from(array[13]) << 16
  | u128::from(array[14]) << 8
  | u128::from(array[15])
}

/// I convert a 8 byte array into a u64 (bigendian)
pub(crate) fn u8_bigendian_slice_to_u64(array: &[u8]) -> u64 {
  u64::from(array[0]) << 56
  | u64::from(array[1]) << 48
  | u64::from(array[2]) << 40
  | u64::from(array[3]) << 32
  | u64::from(array[4]) << 24
  | u64::from(array[5]) << 16
  | u64::from(array[6]) << 8
  | u64::from(array[7])
}

/// I convert a 4 byte array into a u32 (bigendian)
pub(crate) fn u8_bigendian_slice_to_u32(array: &[u8]) -> u32 {
  u32::from(array[0]) << 24
  | u32::from(array[1]) << 16
  | u32::from(array[2]) << 8
  | u32::from(array[3])
}

/// I compute the minimum power of two that is greater or equal to the input
#[inline]
pub(crate) fn min_greater_equal_power_of_two(n: u32) -> u32 {
  2.0f64.powi((n as f64).log2().ceil() as i32) as u32
}

pub(crate) fn u64_to_u32_pair(x: u64) -> (u32, u32) {
  ((x & 0xFFFF_FFFF) as u32, (x >> 32) as u32)
}

pub(crate) fn byte_slice_to_scalar<S: Scalar>(slice: &[u8]) -> S {
  let mut hasher = Sha512::new();
  hasher.input(slice);
  S::from_hash(hasher)
}
/*
// **base58 translation functions**
use num_bigint::{BigInt};
use num_bigint::Sign::Plus;

use num_traits::{Zero};
use crate::errors::ZeiError;
use num_traits::{FromPrimitive,ToPrimitive};
static BASE58_ALPHABET: &'static [u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
static BASE58_INVERSE: [Option<u8>; 128] =  [
    None,     None,     None,     None,     None,     None,     None,     None,//0-7
    None,     None,     None,     None,     None,     None,     None,     None,//8-15
    None,     None,     None,     None,     None,     None,     None,     None,//16-23
    None,     None,     None,     None,     None,     None,     None,     None,//24-31
    None,     None,     None,     None,     None,     None,     None,     None,//32-39
    None,     None,     None,     None,     None,     None,     None,     None,//40-47
    None,     Some(0),  Some(1),  Some(2),  //48  *,1,2,3
    Some(3),  Some(4),  Some(5),  Some(6),  //52  4,5,6,7
    Some(7),  Some(8),  None,     None,     //56  8,9,*,*
    None,     None,     None,     None,     //60
    None,     Some(9),  Some(10), Some(11), //64  *,A,B,C
    Some(12), Some(13), Some(14), Some(15), //68  D,E,F,G
    Some(16), None,     Some(17), Some(18), //72  H,*,J,K
    Some(19), Some(20), Some(21), None,     //76  L,M,N,*
    Some(22), Some(23), Some(24), Some(25), //80  P,Q,R,S,
    Some(26), Some(27), Some(28), Some(29), //84  T,U,V,W
    Some(30), Some(31), Some(32), None,     //88  X,Y,Z,*
    None,     None,     None,     None,     //92
    None,     Some(33), Some(34), Some(35), //96  *,a,b,c
    Some(36), Some(37), Some(38), Some(39), //100 d,e,f,g
    Some(40), Some(41), Some(42), Some(43), //104 h,i,j,k
    None,     Some(44), Some(45), Some(46), //108 *,m,n,o
    Some(47), Some(48), Some(49), Some(50), //112 p,q,r,s
    Some(51), Some(52), Some(53), Some(54), //116 t,u,v,w
    Some(55), Some(56), Some(57), None,     //120 x,y,z,*
    None,     None,     None,     None      //124-127
];

pub(crate) fn to_base58(data: &[u8]) -> String {
    /*
     * I convert a u8 slice @data into a base58 string.
     * @data is read as a Bigendian big integer.
     * Leading zero bytes are replaced with '1' base58 symbol
     *
     */

    let strlen_upper_bound = (1.0 + (data.len() as f64) * 1.4) as usize; // upper bound un log_58(256)
    let mut ret = Vec::with_capacity(strlen_upper_bound);
    let mut leading_zeroes = 0;
    for d in data {
        if *d != 0u8 {
          break;
        }
        leading_zeroes += 1;
    }
    let mut data_as_int = BigInt::from_bytes_be(Plus, data);
    while !data_as_int.is_zero(){
        let quo: BigInt = &data_as_int / 58;
        let x: BigInt = &quo * 58;
        let rem: BigInt = &data_as_int - x;
        data_as_int = quo;
        let (_, bytes) = rem.to_bytes_be();
        let index: u8 = bytes[0];
        ret.push(BASE58_ALPHABET[index as usize]);
    }
    for _ in 0..leading_zeroes{
       ret.push(BASE58_ALPHABET[0]);
    }
    ret.reverse();
    String::from_utf8(ret).unwrap()

}

pub(crate) fn from_base58(data: &str) -> Result<Vec<u8>, ZeiError>  {
    /*
     * I convert a string in base58 format to bigendian vector of bytes.
     * Leading ones base58 chars in original strings are translates to leading 0u8 in results
     *
     */

    let mut big_int: BigInt = Zero::zero();
    let mut factor = BigInt::from_u8(1).unwrap();

    let mut leading_ones = 0;
    for d in data.chars() {
        if d != '1' {
          break;
        }
        leading_ones += 1;
    }

    for d in data.chars().rev(){
        let base58 = match BASE58_INVERSE[d as usize] {
          Some(x) => x as u32,
          None => {
              return Err(ZeiError::BadBase58Format);
          }
        };
        big_int += &factor * base58;
        factor *= 58;
    }

    let mut vec: Vec<u8> = Vec::new();
    while !big_int.is_zero(){
        let byte: BigInt = &big_int % 256;
        big_int = &big_int / 256;
        vec.push(byte.to_u8().unwrap());
    }

    for _ in 0..leading_ones{
        vec.push(0u8);
    }

    vec.reverse();
    Ok(vec)
}

mod test {
    use crate::utils::*;
    #[test]
    fn test_base58_encoding_decoding() {
        let v = vec![1,2,3,4,5];
        let b58_str = to_base58(&v[..]);
        assert_eq!(v, from_base58(&b58_str[..]).unwrap());
    }

    #[test]
    fn test_base58_leading_zeroes() {
        let v = vec![0,1,2,3,4,5];
        let b58_str = to_base58(&v[..]);
        assert_eq!(v, from_base58(&b58_str[..]).unwrap());
    }
}

*/

#[cfg(test)]
mod test {

  #[test]
  fn u32_to_bignedian_u8array() {
    let n: u32 = 0xFA01C673;
    let n_array = super::u32_to_bigendian_u8array(n);
    assert_eq!(0xFA, n_array[0]);
    assert_eq!(0x01, n_array[1]);
    assert_eq!(0xC6, n_array[2]);
    assert_eq!(0x73, n_array[3]);
  }

  #[test]
  fn test_u8_bigendian_slice_to_u32() {
    let array = [0xFA as u8, 0x01 as u8, 0xC6 as u8, 0x73 as u8];
    let n = super::u8_bigendian_slice_to_u32(&array);
    assert_eq!(0xFA01C673, n);
  }

  #[test]
  fn u64_to_bignedian_u8array() {
    let n: u64 = 0xFA01C67322E498A2;
    let n_array = super::u64_to_bigendian_u8array(n);
    assert_eq!(0xFA, n_array[0]);
    assert_eq!(0x01, n_array[1]);
    assert_eq!(0xC6, n_array[2]);
    assert_eq!(0x73, n_array[3]);
    assert_eq!(0x22, n_array[4]);
    assert_eq!(0xE4, n_array[5]);
    assert_eq!(0x98, n_array[6]);
    assert_eq!(0xA2, n_array[7]);
  }

  #[test]
  fn u8_bigendian_slice_to_u64() {
    let array = [0xFA as u8, 0x01 as u8, 0xC6 as u8, 0x73 as u8, 0x22, 0xE4, 0x98, 0xA2];
    let n = super::u8_bigendian_slice_to_u64(&array);
    assert_eq!(0xFA01C67322E498A2, n);
  }

  #[test]
  fn min_greater_equal_power_of_two() {
    assert_eq!(16, super::min_greater_equal_power_of_two(16));
    assert_eq!(16, super::min_greater_equal_power_of_two(15));
    assert_eq!(16, super::min_greater_equal_power_of_two(9));
    assert_eq!(8, super::min_greater_equal_power_of_two(8));
    assert_eq!(8, super::min_greater_equal_power_of_two(6));
    assert_eq!(8, super::min_greater_equal_power_of_two(5));
    assert_eq!(4, super::min_greater_equal_power_of_two(4));
    assert_eq!(4, super::min_greater_equal_power_of_two(3));
    assert_eq!(2, super::min_greater_equal_power_of_two(2));
    assert_eq!(1, super::min_greater_equal_power_of_two(1));
    assert_eq!(0, super::min_greater_equal_power_of_two(0));
  }

  #[test]
  fn u64_to_u32_pair() {
    assert_eq!((32, 0), super::u64_to_u32_pair(32u64));
    assert_eq!((0xFFFFFFFF, 0xFFFFFFFF),
               super::u64_to_u32_pair(0xFFFFFFFFFFFFFFFFu64));
    assert_eq!((0, 0xFFFFFFFF),
               super::u64_to_u32_pair(0xFFFFFFFF00000000u64));
  }
}

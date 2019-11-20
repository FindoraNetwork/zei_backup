//! Zei: Eian's cryptographic library
//#![feature(test)]
//extern crate aes_ctr;

// Skip non useful warnings from clippy
#![allow(clippy::let_and_return)]
#![allow(clippy::many_single_char_names)]

#[macro_use]
extern crate itertools;

#[macro_use] //rustc crushes if not used
extern crate serde_derive;

pub mod algebra;
pub mod api;
pub mod basic_crypto;
pub mod crypto;
pub mod errors;
pub mod serialization;
pub mod setup;
mod utils;
pub mod xfr;

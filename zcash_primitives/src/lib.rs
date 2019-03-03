#[macro_use]
extern crate failure;

#[macro_use]
extern crate lazy_static;

extern crate blake2_rfc;
extern crate byteorder;
extern crate ff;
extern crate hex;
extern crate pairing;
extern crate rand;
extern crate sapling_crypto;
extern crate sha2;

use sapling_crypto::jubjub::JubjubBls12;

pub mod block;
pub mod merkle_tree;
pub mod sapling;
mod serialize;
pub mod transaction;

lazy_static! {
    pub static ref JUBJUB: JubjubBls12 = { JubjubBls12::new() };
}

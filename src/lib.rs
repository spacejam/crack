#![allow(unused)]
extern crate rustc_serialize;
extern crate bincode;
extern crate time;
extern crate crossbeam;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingResult};
use rustc_serialize::{Encodable, Decodable};

mod clock;
mod crc16;

pub mod transport;
pub mod kernel;

pub use clock::{Clock, RealClock, TestClock};
use crc16::{crc16_arr, crc16};

#[inline(always)]
pub fn to_binary<T: Encodable>(s: &T) -> Vec<u8> {
    encode(s, SizeLimit::Infinite).unwrap()
}

#[inline(always)]
pub fn to_framed_binary<T: Encodable>(s: &T) -> Vec<u8> {
    let mut bytes = to_binary(s);
    let mut size = usize_to_array(bytes.len()).to_vec();
    let mut ret = Vec::with_capacity(bytes.len() + 4);
    ret.append(&mut size);
    ret.append(&mut bytes);
    ret
}

#[inline(always)]
pub fn from_binary<T: Decodable>(encoded: Vec<u8>) -> DecodingResult<T> {
    decode(&encoded[..])
}

#[inline(always)]
pub fn usize_to_array(u: usize) -> [u8; 4] {
    [(u >> 24) as u8, (u >> 16) as u8, (u >> 8) as u8, u as u8]
}

#[inline(always)]
pub fn array_to_usize(ip: [u8; 4]) -> usize {
    ((ip[0] as usize) << 24) as usize + ((ip[1] as usize) << 16) as usize +
    ((ip[2] as usize) << 8) as usize + (ip[3] as usize)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

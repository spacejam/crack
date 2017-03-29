#![allow(unused)]
extern crate bytes;
extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate bincode;
extern crate time;
extern crate crossbeam;
extern crate tokio_core;
extern crate futures;
extern crate serde_json;
extern crate rand;
extern crate tokio_timer;
extern crate tokio_io;
extern crate uuid;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingResult};
use rustc_serialize::{Encodable, Decodable};

mod clock;
mod crc16;

// TODO
// let rx = "127.0.0.1:8080".inbox<Msgtype>().unwrap();
// let tx = "1.2.3.4:5".outbox<MsgType>();
// means Into<Peer> for SockAddr

#[macro_export]
macro_rules! codec_boilerplate {
    ($($T:ty),*) => {
        $(
            impl Decoder for $T {
                type Item = $T;
                type Error = io::Error;

                fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<$T>> {
                    println!("decoding message");
                    decode(&buf.take()[..])
                        .map(|v| {
                            println!("successfully decoded {:?}", v);
                            Some(v)
                        })
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                }
            }

            impl Encoder for $T {
                type Item = $T;
                type Error = io::Error;

                fn encode(&mut self, msg: $T, buf: &mut BytesMut) -> io::Result<()> {
                    println!("encoding message");
                    buf.put(encode(&msg, SizeLimit::Infinite).unwrap());
                    Ok(())
                }
            }
        )*
    };
}

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

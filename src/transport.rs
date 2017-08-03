use std::convert;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::iter;
use std::rc::Rc;
use std::str;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::thread;

use crossbeam::sync::MsQueue;
use futures::sync::mpsc;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::{Core, Handle};
use rustc_serialize::{Encodable, Decodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingResult};
use serde_json;
use bytes::{BufMut, BytesMut};
use tokio_io::codec::{Encoder, Decoder, length_delimited};
use tokio_io::AsyncRead;
use futures::{future, sink, Sink, Stream, Future, IntoFuture};
use futures::stream::{self, SplitSink, SplitStream};
use uuid::Uuid;

use {to_binary, from_binary};
use kernel::Peer;

trait Transport<Addr, Rx, Tx>: Sized {
    fn rx(&self) -> Arc<MsQueue<(Addr, Rx)>>;
    fn tx(&self) -> Arc<MsQueue<(Addr, Tx)>>;
    fn shutdown(self) {}
}

struct TCPTransport<Rx, Tx> {
    rx: Arc<MsQueue<(SocketAddr, Rx)>>,
    tx: Arc<MsQueue<(SocketAddr, Tx)>>,
    connections: Arc<RwLock<HashMap<SocketAddr, mpsc::UnboundedSender<Tx>>>>,
}

impl<Rx, Tx> TCPTransport<Rx, Tx>
    where Rx: Decodable + Send + 'static,
          Tx: Encodable + Send + 'static
{
    pub fn new(addr: SocketAddr) -> TCPTransport<Rx, Tx> {
        let msgs = Arc::new(MsQueue::new());
        let msgs2 = msgs.clone();

        let connections = Arc::new(RwLock::new(HashMap::new()));
        let connections2 = connections.clone();

        thread::spawn(move || {
            loop {
                let (addr, msg): (SocketAddr, Tx) = msgs2.pop();
                let connections = connections2.read().unwrap();
                if let Some(tx) = connections.get(&addr) {
                    let tx: &mpsc::UnboundedSender<Tx> = tx;
                    tx.send(msg).unwrap();
                    continue;
                }
                drop(connections);

                let tx = send(addr.clone());
                tx.clone().send(msg).wait().unwrap();
                let mut connections = connections2.write().unwrap();
                connections.insert(addr, tx);
            }
        });

        TCPTransport {
            rx: recv(addr),
            tx: msgs.clone(),
            connections: connections.clone(),
        }
    }
}

impl<Rx, Tx> Transport<SocketAddr, Rx, Tx> for TCPTransport<Rx, Tx> {
    fn rx(&self) -> Arc<MsQueue<(SocketAddr, Rx)>> {
        self.rx.clone()
    }

    fn tx(&self) -> Arc<MsQueue<(SocketAddr, Tx)>> {
        self.tx.clone()
    }
}

fn recv<T>(addr: SocketAddr) -> Arc<MsQueue<(SocketAddr, T)>>
    where T: Decodable + Send + 'static
{

    let msgs = Arc::new(MsQueue::new());
    let msgs2 = msgs.clone();

    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let listener = TcpListener::bind(&addr, &handle).unwrap();

        let server = listener.incoming().for_each(move |(socket, addr)| {
            println!("got a new connection from {:?}", addr);
            let msgs3 = msgs2.clone();

            let deframed = length_delimited::FramedRead::new(socket);
            let rx = deframed.map_err(|e| panic!(e))
                .and_then(|buf| decode(&buf).map_err(|e| panic!(e)));

            let pusher = rx.for_each(move |msg| {
                    msgs3.push((addr, msg));
                    Ok(())
                })
                .into_future()
                .then(|_| Ok(()));

            handle.spawn(pusher);

            Ok(())
        });

        core.run(server).unwrap();
    });

    msgs
}

fn send<T>(addr: SocketAddr) -> mpsc::UnboundedSender<T>
    where T: Encodable + Send + 'static
{
    let (outbound_tx, outbound_rx) = mpsc::unbounded();

    thread::spawn(move || {

        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let tcp = TcpStream::connect(&addr, &handle);

        let client = tcp.and_then(move |socket| {
            let framed = length_delimited::FramedWrite::new(socket);
            outbound_rx.map(|msg| encode(&msg, SizeLimit::Infinite).unwrap())
                .map_err(|e| panic!(e))
                .forward(framed)
        });

        // let reconnector = future::loop_fn(client, |client| client);

        core.run(client).unwrap();

        println!("sender exiting");

    });

    outbound_tx
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct Msg {
    inner: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let rx = recv::<Msg>("127.0.0.1:8080".parse().unwrap());
        thread::sleep_ms(300);
        let ref mut tx = &mut send::<Msg>("127.0.0.1:8080".parse().unwrap());
        for i in 1..4 {
            let msg = Msg { inner: i };
            println!("sending msg: {:?}", msg);
            tx.send(msg).wait().unwrap();
        }

        for i in 1..4 {
            println!("blocking on pop");
            let (a, m) = rx.pop();
            println!("rtt {:?} from {:?}", m, a);

        }
    }
}

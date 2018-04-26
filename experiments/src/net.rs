extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::env;
use std::io::{Error, ErrorKind, BufReader};
use std::net::SocketAddr;
use std::thread;

use rustc_serialize::{Encodable, Decodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingResult};

use futures::Future;
use futures::stream::{self, Stream};
use futures::sync::mpsc;
use tokio_io::codec::length_delimited;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::io;
use tokio_io::AsyncRead;

//  tx/rx setup
//    add to map
//  sending (addr, msg)
//    if not connected, chain a connection + tx/rx setup
//    forward msg to connections[addr] tx
//  receiving connection (addr)
//    tx/rx setup
//  receiving (addr, msg)
//    outgoing_tx <- (addr, msg)

// sending to unconnected server: connect, transport, tx, forward

// receiving new connection: set up its transport + tx, add to core

// receiving old connection:

fn transport<T>
    (raw_addr: String)
     -> (mpsc::UnboundedSender<(SocketAddr, T)>, mpsc::UnboundedReceiver<(SocketAddr, T)>)
    where T: Encodable + Decodable + Send + 'static
{
    let addr = raw_addr.parse::<SocketAddr>().unwrap();
    let (incoming_tx, incoming_rx) = mpsc::unbounded();
    let (outgoing_tx, outgoing_rx) = mpsc::unbounded();

    thread::spawn(move || {
        // Create the event loop and TCP listener we'll accept connections on.
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let socket = TcpListener::bind(&addr, &handle).unwrap(); // TODO graceful shutdown
        info!("Listening on: {}", addr);

        // This is a single-threaded server, so we can just use Rc and RefCell to
        // store the map of all connections we know about.
        let connections = Rc::new(RefCell::new(HashMap::new()));

        let srv = socket.incoming().for_each(move |(stream, addr)| {
            debug!("New Connection: {}", addr);
            let (reader, writer) = stream.split();
            let framed_reader = length_delimited::FramedRead::new(reader);
            let framed_writer = length_delimited::FramedWrite::new(writer);

            // Create a channel for our stream, which other sockets will use to
            // send us messages. Then register our address with the stream to send
            // data to us.
            let (tx, rx) = mpsc::unbounded();
            connections.borrow_mut().insert(addr, tx);

            // Define here what we do for the actual I/O. That is, read a bunch of
            // lines from the socket and dispatch them while we also write any lines
            // from other sockets.
            let connections_inner = connections.clone();

            // Model the read portion of this socket by mapping an infinite
            // iterator to each line off the socket. This "loop" is then
            // terminated with an error once we hit EOF on the socket.
            let iter = stream::iter(iter::repeat(()).map(Ok::<(), Error>));
            let socket_reader = iter.fold(framed_reader, move |reader, _| {
                // Read a line off the socket, failing if we're at EOF
                let msg = io::read_to_end(reader, Vec::new());
                let msg = msg.and_then(|(reader, vec)| {
                    if vec.len() == 0 {
                        Err(Error::new(ErrorKind::BrokenPipe, "broken pipe"))
                    } else {
                        Ok((reader, vec))
                    }
                });

                // Convert the bytes we read into a string, and then send that
                // string to all other connected clients.
                let msg = msg.map(|(reader, vec)| (reader, decode(&vec[..]).map_err(|_| ())));
                let connections = connections_inner.clone();
                msg.map(move |(reader, message)| {
                    trace!("{}: {:?}", addr, message);
                    let mut conns = connections.borrow_mut();
                    if let Ok(msg) = message {
                        // For each open connection except the sender, send the
                        // string via the channel.
                        let iter = conns.iter_mut()
                            .filter(|&(&k, _)| k != addr)
                            .map(|(_, v)| v);
                        for tx in iter {
                            tx.send(format!("{}: {}", addr, msg)).unwrap();
                        }
                    } else {
                        let tx = conns.get_mut(&addr).unwrap();
                        tx.send("You didn't send valid UTF-8.".to_string()).unwrap();
                    }
                    reader
                })
            });

            // Whenever we receive a string on the Receiver, we write it to
            // `WriteHalf<TcpStream>`.
            let socket_writer = rx.fold(writer, |writer, msg| {
                let amt = io::write_all(writer, msg.into_bytes());
                let amt = amt.map(|(writer, _)| writer);
                amt.map_err(|_| ())
            });

            // Now that we've got futures representing each half of the socket, we
            // use the `select` combinator to wait for either half to be done to
            // tear down the other. Then we spawn off the result.
            let connections = connections.clone();
            let socket_reader = socket_reader.map_err(|_| ());
            let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
            handle.spawn(connection.then(move |_| {
                connections.borrow_mut().remove(&addr);
                debug!("Connection {} closed.", addr);
                Ok(())
            }));

            Ok(())
        });

        // execute server
        core.run(srv).unwrap();
    });

    (outgoing_tx, incoming_rx)
}

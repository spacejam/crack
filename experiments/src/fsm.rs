#[macro_export]
macro_rules! fsm {
    (inner: $INNER:ty, transitions: [($($FROM:ty => ($($TO:ty),*)),*)]) => {
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

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn it_works() {
        struct Inner {
            inner: u64,
        }

        // fsm!{
        // inner: Inner,
        // transitions: [
        // A => [A, B, C],
        // B => [A, C],
        // C => [C],
        // ]
        // }
        //
    }
}

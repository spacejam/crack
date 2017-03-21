pub enum PeerState {
    Connected,
    Disconnected,
}

pub struct Peer {
    state: PeerState,
    addr: String,
}

pub trait Kernel {
    fn peers(&self) -> Vec<Peer>;
    fn ping(&self, Peer) -> Result<(), ()>;
    fn purge(&self, Peer);
}

pub struct K {
    peers: Vec<Peer>,
}

impl Default for K {
    fn default() -> K {
        K { peers: vec![] }
    }
}

impl K {
    fn cron(&self) {
        // for
    }
}

impl Kernel for K {
    fn peers(&self) -> Vec<Peer> {
        vec![]
    }

    fn ping(&self, peer: Peer) -> Result<(), ()> {
        Ok(())
    }

    fn purge(&self, peer: Peer) {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

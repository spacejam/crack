use crossbeam::sync::MsQueue;

use kernel::Peer;

trait Transport<Req, Res, E>: Sized {
    fn new(listen: String) -> Result<Self, E>;
    fn rx(&self) -> MsQueue<(Peer, Res)>;
    fn tx(&self) -> MsQueue<(Peer, Req)>;
    fn shutdown(self);
}

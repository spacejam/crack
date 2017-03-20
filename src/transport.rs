use crossbeam::sync::MsQueue;

trait Transport<Req, Res> {
    fn new(listen: String) -> Self;
    fn sender(&self) 
}

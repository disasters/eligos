pub trait SendChannel<M: Send, E> {
    fn send_msg(&self, msg: M) -> E;
}

impl<M: Send> SendChannel<M, Result<(), NotifyError<M>>> for mio::Sender<M> {
    fn send_msg(&self, msg: M) -> Result<(), NotifyError<M>> {
        self.send(msg)
    }
}

impl<M: Send> SendChannel<M, Result<(), SendError<M>>> for Sender<M> {
    fn send_msg(&self, msg: M) -> Result<(), SendError<M>> {
        self.send(msg)
    }
}

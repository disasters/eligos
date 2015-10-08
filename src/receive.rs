pub trait Receive<Req, Res> {
    fn receive(&self, &Req) -> Option<Res>;
}

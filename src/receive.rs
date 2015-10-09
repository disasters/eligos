use ::ClientInfo;

pub trait Receive<Req, Res> {
    fn receive(&mut self, ClientInfo, &Req) -> Option<Res>;
}

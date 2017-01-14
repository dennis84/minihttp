extern crate futures;
extern crate httparse;
extern crate net2;
extern crate time;
extern crate tokio_core;
extern crate tokio_proto;

mod date;
mod request;
mod response;

use std::io;

pub use request::Request;
pub use response::Response;

use tokio_proto::streaming::pipeline::{Frame, ServerProto};
use tokio_core::io::{Io, Codec, Framed, EasyBuf};

pub struct Http;

impl<T: Io + 'static> ServerProto<T> for Http {
    type Request = Request;
    type RequestBody = String;
    type Response = Response;
    type ResponseBody = String;
    type Error = io::Error;

    type Transport = Framed<T, HttpCodec>;
    type BindTransport = io::Result<Framed<T, HttpCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, HttpCodec>> {
        Ok(io.framed(HttpCodec))
    }
}

pub struct HttpCodec;

impl Codec for HttpCodec {
    type In = Frame<Request, String, io::Error>;
    type Out = Frame<Response, String, io::Error>;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        match request::decode(buf) {
            Ok(Some(req)) => Ok(Some(Frame::Message {
                message: req,
                body: true
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e)
        }
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        match msg {
            Frame::Message { mut message, body } => {
                if body == false {
                    let length = message.body.len();
                    message.header("Content-Length", &length.to_string());
                }

                response::encode(message, buf);
            },
            Frame::Body { chunk } => {
                match chunk {
                    Some(x) => response::encode_chunk(x, buf),
                    None => {},
                }
            },
            Frame::Error { error } => {},
        }

        Ok(())
    }
}

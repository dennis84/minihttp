extern crate futures;
extern crate minihttp;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use futures::{future, Future};
use minihttp::{Request, Response, Http};
use tokio_proto::streaming::{Body, Message};
use tokio_proto::TcpServer;
use tokio_service::Service;

struct HelloService {}

impl Service for HelloService {
    type Request = Message<Request, Body<String, Self::Error>>;
    type Response = Message<Response, Body<String, Self::Error>>;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    type Error = io::Error;

    fn call(&self, _: Self::Request) -> Self::Future {
        let mut resp = Response::new();
        resp.body("Hello World");
        future::ok(Message::WithoutBody(resp)).boxed()
    }
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let srv = TcpServer::new(Http, addr);
    srv.serve(|| Ok(HelloService {}));
}

extern crate futures;
extern crate minihttp;
extern crate tokio_core;
extern crate tokio_proto;
extern crate service_fn;

use std::io;
use futures::*;
use minihttp::{Request, Response, Http};
use tokio_proto::streaming::{Body, Message};
use tokio_proto::TcpServer;
use service_fn::service_fn;

type RequestMessage = Message<Request, Body<String, io::Error>>;
type ResponseMessage = Message<Response, Body<String, io::Error>>;
type FutureResponse = Box<Future<Item = ResponseMessage, Error = io::Error>>;

fn hello_world() -> FutureResponse {
    let mut resp = Response::new();
    resp.body("Hello World");
    future::ok(Message::WithoutBody(resp)).boxed()
}

fn not_found() -> FutureResponse {
    let mut resp = Response::new();
    resp.status_code(404, "Not Found");
    future::ok(Message::WithoutBody(resp)).boxed()
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let srv = TcpServer::new(Http, addr);

    srv.serve(|| Ok(service_fn(move |req: RequestMessage| {
        match req.into_inner().path() {
            "/" => hello_world(),
            _ => not_found(),
        }
    })));
}

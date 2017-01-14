extern crate futures;
extern crate minihttp;
extern crate tokio_core;
extern crate tokio_proto;
extern crate service_fn;

use std::io;
use futures::*;
use minihttp::{Request, Response, Http};
use tokio_core::reactor::Remote;
use tokio_proto::streaming::{Body, Message};
use tokio_proto::TcpServer;
use service_fn::service_fn;

type RequestMessage = Message<Request, Body<String, io::Error>>;
type ResponseMessage = Message<Response, Body<String, io::Error>>;
type FutureResponse = Box<Future<Item = ResponseMessage, Error = io::Error>>;

fn events(remote: Remote) -> FutureResponse {
    let (tx, rx) = Body::pair();
    let stream = tx.send(Ok("data: a\r\n".to_string()))
        .and_then(|tx| tx.send(Ok("data: b\r\n".to_string())))
        .and_then(|tx| tx.send(Ok("data: c\r\n".to_string())))
        .map(|_| {()})
        .map_err(|_| ())
        .boxed();

    remote.spawn(move |_| stream);
    let mut resp = Response::new();
    resp.header("Content-Type", "text/event-stream");
    resp.header("Cache-Control", "no-cache");

    future::ok(Message::WithBody(resp, rx)).boxed()
}

fn not_found() -> FutureResponse {
    let mut resp = Response::new();
    resp.status_code(404, "Not Found");
    future::ok(Message::WithoutBody(resp)).boxed()
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let srv = TcpServer::new(Http, addr);

    srv.with_handle(|handle| {
        let remote = handle.remote().clone();
        move || {
            let remote1 = remote.clone();
            Ok(service_fn(move |req: RequestMessage| {
                match req.into_inner().path() {
                    "/events" => events(remote1.clone()),
                    _ => not_found(),
                }
            }))
        }
    });
}

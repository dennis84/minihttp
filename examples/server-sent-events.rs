extern crate time;
extern crate futures;
extern crate minihttp;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_timer;
extern crate service_fn;

use std::io;
use std::time::*;
use futures::*;
use minihttp::{Request, Response, Http};
use tokio_core::reactor::Remote;
use tokio_proto::streaming::{Body, Message};
use tokio_proto::TcpServer;
use tokio_timer::*;
use service_fn::service_fn;

type RequestMessage = Message<Request, Body<String, io::Error>>;
type ResponseMessage = Message<Response, Body<String, io::Error>>;
type FutureResponse = Box<Future<Item = ResponseMessage, Error = io::Error>>;

fn events(remote: Remote) -> FutureResponse {
    let (tx, rx) = Body::pair();
    let timer = Timer::default();
    let interval = timer.interval(Duration::from_millis(2000));
    let stream = interval
        .map_err(|_| ())
        .fold(tx, |t, _| {
            let data = format!("data: {}\r\n", time::get_time().sec);
            t.send(Ok(data.to_string())).map_err(|_| ())
        })
        .map(|_| ());

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

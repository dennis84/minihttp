extern crate time;
extern crate futures;
extern crate minihttp;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_timer;

use std::io;
use std::time::Duration;
use futures::{future, Future, Stream, Sink};
use minihttp::{Request, Response, Http};
use tokio_core::reactor::Remote;
use tokio_proto::streaming::{Body, Message};
use tokio_proto::TcpServer;
use tokio_service::Service;
use tokio_timer::Timer;

struct EventService {
    remote: Remote,
}

impl Service for EventService {
    type Request = Message<Request, Body<String, Self::Error>>;
    type Response = Message<Response, Body<String, Self::Error>>;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    type Error = io::Error;

    fn call(&self, _: Self::Request) -> Self::Future {
        let (tx, rx) = Body::pair();
        let timer = Timer::default();
        let interval = timer.interval(Duration::from_millis(2000));
        let stream = interval
            .map_err(|_| ())
            .fold(tx, |t, _| {
                let data = format!("data: {}\n\n", time::get_time().sec);
                t.send(Ok(data.to_string())).map_err(|_| ())
            })
            .map(|_| ());

        self.remote.spawn(|_| stream);
        let mut resp = Response::new();
        resp.header("Content-Type", "text/event-stream");
        resp.header("Cache-Control", "no-cache");

        future::ok(Message::WithBody(resp, rx)).boxed()
    }
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let srv = TcpServer::new(Http, addr);
    srv.with_handle(|handle| {
        let remote = handle.remote().clone();
        move || Ok(EventService {
            remote: remote.clone(),
        })
    });
}

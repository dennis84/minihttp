extern crate time;
extern crate futures;
extern crate minihttp;
extern crate tokio_proto;
extern crate tokio_timer;
extern crate tokio_service;

use std::{io, thread};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use futures::{future, Future, Sink};
use futures::sync::mpsc::Sender;
use minihttp::{Request, Response, Http};
use tokio_proto::streaming::{Body, Message};
use tokio_proto::TcpServer;
use tokio_service::Service;

type BodySender = Sender<Result<String, io::Error>>;

struct EventService {
    connections: Arc<Mutex<Vec<BodySender>>>,
}

impl Service for EventService {
    type Request = Message<Request, Body<String, Self::Error>>;
    type Response = Message<Response, Body<String, Self::Error>>;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    type Error = io::Error;

    fn call(&self, _: Self::Request) -> Self::Future {
        let (tx, rx) = Body::pair();
        self.connections.lock().unwrap().push(tx);

        let mut resp = Response::new();
        resp.header("Content-Type", "text/event-stream");
        resp.header("Cache-Control", "no-cache");
        future::ok(Message::WithBody(resp, rx)).boxed()
    }
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let srv = TcpServer::new(Http, addr);

    let connections: Arc<Mutex<Vec<BodySender>>> = Arc::new(Mutex::new(Vec::new()));
    let connections_inner = connections.clone();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(2000));

        let data = format!("data: {}\n\n", time::get_time().sec);
        let mut conns = connections.lock().unwrap();

        println!("Send data to {} connections.", conns.len());
        *conns = conns.iter_mut().filter_map(|tx| {
            match tx.send(Ok(data.to_string())).wait() {
                Ok(_) => Some(tx.to_owned()),
                Err(_) => None,
            }
        }).collect::<Vec<_>>();
    });

    println!("Listening on http://{}", addr);
    srv.serve(move || Ok(EventService {
        connections: connections_inner.clone(),
    }));
}

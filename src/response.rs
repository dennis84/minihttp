use std::fmt::{self, Write};

pub struct Response {
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub status_code: StatusCode,
}

pub enum StatusCode {
    Ok,
    Custom(u32, String)
}

impl Response {
    pub fn new() -> Response {
        Response {
            headers: Vec::new(),
            body: String::new(),
            status_code: StatusCode::Ok,
        }
    }

    pub fn status_code(&mut self, code: u32, message: &str) -> &mut Response {
        self.status_code = StatusCode::Custom(code, message.to_string());
        self
    }

    pub fn header(&mut self, name: &str, val: &str) -> &mut Response {
        self.headers.push((name.to_string(), val.to_string()));
        self
    }

    pub fn body(&mut self, s: &str) -> &mut Response {
        self.body = s.to_string();
        self
    }
}

pub fn encode(msg: Response, buf: &mut Vec<u8>) {
    let now = ::date::now();

    write!(FastWrite(buf), "\
        HTTP/1.1 {}\r\n\
        Server: MiniHTTP\r\n\
        Date: {}\r\n\
    ", msg.status_code, now).unwrap();

    for &(ref k, ref v) in &msg.headers {
        buf.extend_from_slice(k.as_bytes());
        buf.extend_from_slice(b": ");
        buf.extend_from_slice(v.as_bytes());
        buf.extend_from_slice(b"\r\n");
    }

    buf.extend_from_slice(b"\r\n");
    buf.extend_from_slice(msg.body.as_bytes());
}

pub fn encode_chunk(msg: String, buf: &mut Vec<u8>) {
    buf.extend_from_slice(msg.as_bytes());
    buf.extend_from_slice(b"\r\n");
}

// TODO: impl fmt::Write for Vec<u8>
//
// Right now `write!` on `Vec<u8>` goes through io::Write and is not super
// speedy, so inline a less-crufty implementation here which doesn't go through
// io::Error.
struct FastWrite<'a>(&'a mut Vec<u8>);

impl<'a> fmt::Write for FastWrite<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        fmt::write(self, args)
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StatusCode::Ok => f.pad("200 OK"),
            StatusCode::Custom(c, ref s) => write!(f, "{} {}", c, s),
        }
    }
}

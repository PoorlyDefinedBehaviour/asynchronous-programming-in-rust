use crate::future::{Future, PollState};
use crate::runtime;
use mio::{Interest, Token};
use std::io::{ErrorKind, Read, Write};

pub struct Http;

impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

pub struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    fn new(path: &str) -> Self {
        Self {
            stream: None,
            buffer: vec![],
            path: path.to_owned(),
        }
    }

    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);
        stream.write_all(get_req(&self.path).as_bytes()).unwrap();
        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        if self.stream.is_none() {
            self.write_request();
            runtime::registry()
                .register(self.stream.as_mut().unwrap(), Token(0), Interest::READABLE)
                .unwrap();
        }
        let mut buffer = vec![0_u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buffer) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer).to_string();
                    return PollState::Ready(s);
                }
                Ok(n) => {
                    self.buffer.extend(&buffer[0..n]);
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => return PollState::Pending,
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) => panic!("reading from stream: {err:?}"),
            }
        }
    }
}

fn get_req(path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
}

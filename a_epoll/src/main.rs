use std::{
    io::{Read, Write},
    net::TcpStream,
};

use poll::Poll;

mod ffi;
mod poll;

// Start the API: cargo r --bin delayserver
// Start the client: cargo r --bin main
fn main() -> std::io::Result<()> {
    let mut poll = Poll::new()?;
    let n_events = 5;
    let mut streams = Vec::new();
    let addr = "localhost:8001";
    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let mut stream = std::net::TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        stream.write_all(request.as_bytes())?;
        poll.registry()
            .register(&stream, i, ffi::EPOLLIN | ffi::EPOLLET)?;
        streams.push(stream);
    }

    let mut handled_events = 0;

    while handled_events < n_events {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;
        if events.is_empty() {
            println!("TIMEOUT (OR SPURIOUS EVENT NOTIFICATION)");
            continue;
        }
        handled_events += handle_events(&events, &mut streams)?;
    }

    println!("FINISHED");
    Ok(())
}

fn get_req(path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
}

fn handle_events(events: &[ffi::Event], streams: &mut [TcpStream]) -> std::io::Result<usize> {
    let mut handled_events = 0;

    for event in events {
        let index = event.token();
        let mut data = vec![0_u8; 4096];
        loop {
            match streams[index].read(&mut data) {
                Ok(0) => {
                    handled_events += 1;
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    println!("RECEIVED: {:?}", event);
                    println!("{txt}\n------\n");
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => return Err(err),
            }
        }
    }

    Ok(handled_events)
}

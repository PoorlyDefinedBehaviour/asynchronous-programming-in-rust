mod future;
mod http;

use std::time::Duration;

use future::{Future, PollState};

use crate::http::Http;

struct Coroutine {
    state: State,
}

enum State {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("program starting");
                    self.state = State::Wait1(Box::new(Http::get("/600/HelloWorld1")));
                }
                State::Wait1(ref mut fut) => match fut.poll() {
                    PollState::Pending => return PollState::Pending,
                    PollState::Ready(text) => {
                        println!("wait 1 text: {text}");
                        self.state = State::Wait2(Box::new(Http::get("/400/HelloWorld2")));
                    }
                },
                State::Wait2(ref mut fut) => match fut.poll() {
                    PollState::Pending => return PollState::Pending,
                    PollState::Ready(text) => {
                        println!("wait 2 text: {text}");
                        self.state = State::Resolved;
                        return PollState::Ready(());
                    }
                },
                State::Resolved => panic!("polled resolved future"),
            }
        }
    }
}

fn main() {
    let mut future = Coroutine::new();

    loop {
        match future.poll() {
            PollState::Pending => {
                println!("schedule other tasks");
            }
            PollState::Ready(()) => {
                println!("future resolved");
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

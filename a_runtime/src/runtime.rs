use crate::future::{Future, PollState};
use mio::{Events, Poll, Registry};
use std::sync::OnceLock;

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() &'static Registry {
  REGISTRY.get().expect("called outside a runtime context")
}

pub struct Runtime {
  poll: Poll,
}

impl Runtime {
  pub fn new() -> Self {
    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();
    REGISTRY.set(registry).unwrap();
    Self { poll }
  }

  pub fn block_on<F>(&mut self, future: F) 
  where F: Future<Output = String> {
    let mut future = future;
    loop {
      match future.poll() {
        PollState::Pending => {
          println!("Schedule other tasks");
          let mut events = Events::with_capacity(100);
          self.poll.poll(&mut events, None).unwrap();
        }
        PollState::Ready => {
          break;
        }
      }
    }
  }
}
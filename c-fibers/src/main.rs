#![feature(naked_functions)]

use std::{arch::asm, char::MAX};

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;

struct Runtime {
    threads: Vec<GreenThread>,
    /// The threads that's currently running.
    current: usize,
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    /// Thread is ready to be assigned a task.
    Available,
    /// Thread is running.
    Running,
    // Thread is ready to resume execution.
    Ready,
}

struct GreenThread {
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

/// The data our CPU needs to resume where it left off.
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

impl Runtime {
    fn new() -> Self {
        let base_thread = GreenThread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };

        let mut threads = Vec::with_capacity(MAX_THREADS + 1);

        threads.push(base_thread);

        for _ in 0..MAX_THREADS {
            threads.push(GreenThread::new());
        }

        Self {
            threads,
            current: 0,
        }
    }

    fn init(&self) {
        unsafe {
            RUNTIME = self as *const Runtime as usize;
        }
    }

    fn run(&mut self) -> ! {
        while self.schedule_thread() {}
        std::process::exit(0);
    }

    #[inline(never)]
    fn schedule_thread(&mut self) -> bool {
        let mut pos = self.current;

        while self.threads[pos].state != State::Ready {
            pos += 1;

            if pos == self.threads.len() {
                pos = 0;
            }

            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
            let new: *const ThreadContext = &self.threads[pos].ctx;
            asm!("call switch", in ("rdi") old, in("rsi") new, clobber_abi("C"));
        }

        !self.threads.is_empty()
    }

    fn thread_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.schedule_thread();
        }
    }

    fn spawn(&mut self, f: fn()) {
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("no available thread");

        let stack_size = available.stack.len();

        unsafe {
            let stack_ptr = available.stack.as_mut_ptr().add(stack_size);
            let stack_ptr = (stack_ptr as usize & !15) as *mut u8;
            std::ptr::write(stack_ptr.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(stack_ptr.offset(-24) as *mut u64, skip as u64);
            std::ptr::write(stack_ptr.offset(-32) as *mut u64, f as u64);
            available.ctx.rsp = stack_ptr.offset(-32) as u64;
        }

        available.state = State::Ready;
    }
}

impl GreenThread {
    fn new() -> Self {
        Self {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
        }
    }
}

fn guard() {
    unsafe {
        let runtime_ptr = RUNTIME as *mut Runtime;
        (*runtime_ptr).thread_return();
    }
}

#[naked]
unsafe extern "C" fn skip() {
    asm!("ret", options(noreturn))
}

fn yield_thread() {
    unsafe {
        let runtime_ptr = RUNTIME as *mut Runtime;
        (*runtime_ptr).schedule_thread();
    }
}

#[naked]
#[no_mangle]
unsafe extern "C" fn switch() {
    asm!(
        "mov [rdi +0x00], rsp",
        "mov [rdi +0x08], r15",
        "mov [rdi +0x10], r14",
        "mov [rdi +0x18], r13",
        "mov [rdi +0x20], r12",
        "mov [rdi +0x28], rbx",
        "mov [rdi +0x30], rbp",
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "ret",
        options(noreturn)
    )
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    runtime.spawn(|| {
        println!("THREAD 1 STARTING");

        let id = 1;

        for i in 0..10 {
            println!("thread: {id} counter: {i}");
            yield_thread();
        }

        println!("THREAD 1 FINISHED");
    });

    runtime.spawn(|| {
        println!("THREAD 2 STARTING");

        let id = 2;

        for i in 0..15 {
            println!("thread: {id} counter {i}");
            yield_thread();
        }

        println!("THREAD 2 FINISHED");
    });

    runtime.run();
}

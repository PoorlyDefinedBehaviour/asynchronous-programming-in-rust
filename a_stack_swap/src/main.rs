use core::arch::asm;

const STACK_SIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    /// Stack pointer.
    rsp: u64,
}

fn hello() -> ! {
    println!("I LOVE WAKING UP ON A NEW STACK!");
    loop {}
}

unsafe fn gt_switch(new: *const ThreadContext) {
    asm!(
      "mov rsp, [{0} + 0x00]",
      "ret",
      in(reg) new
    )
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; STACK_SIZE as usize];

    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(STACK_SIZE);
        let stack_bottom_aligned = (stack_bottom as usize & !15) as *mut u8;
        std::ptr::write(stack_bottom_aligned.offset(-16) as *mut u64, hello as u64);
        ctx.rsp = stack_bottom_aligned.offset(-16) as u64;
        gt_switch(&ctx);
    }
}

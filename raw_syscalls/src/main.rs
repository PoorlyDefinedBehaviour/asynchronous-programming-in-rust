use std::arch::asm;

#[inline(never)]
fn syscall(message: &str) {
    let msg_ptr = message.as_ptr();
    let len = message.len();
    unsafe {
        asm!(
          "mov rax, 1",
          "mov rdi, 1",
          "syscall",
          in("rsi") msg_ptr,
          in("rdx") len,
          out("rax") _,
          out("rdi") _,
          lateout("rsi") _,
          lateout("rdx") _
        )
    }
}

fn main() {
    let message = "Hello world from raw syscall!\n";
    syscall(message);
    // Hello world from raw syscall!
}

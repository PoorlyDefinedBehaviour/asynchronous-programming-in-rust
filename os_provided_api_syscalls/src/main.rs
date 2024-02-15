#[cfg(target_family = "unix")]
#[link(name = "c")]
extern "C" {
    fn write(fd: u32, buf: *const u8, count: usize) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetStdHandle(nStdHandle: i32) -> i32;
    fn WriteConsoleW(
        hConsoleOutput: i32,
        lpBuffer: *const u16,
        numberOfCharsToWrite: u32,
        lpNumberOfCharsWritten: *mut u32,
        lpReserved: *const std::ffi::c_void,
    ) -> i32;
}

#[cfg(target_family = "unix")]
fn syscall(message: &str) -> std::io::Result<()> {
    let msg_ptr = message.as_ptr();
    let len = message.len();
    let res = unsafe { write(1, msg_ptr, len) };
    if res == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn syscall(message: &str) -> std::io::Result<()> {
    let msg: Vec<u16> = message.encode_utf16().collect();
    let msg_ptr = msg.as_ptr();
    let len = msg.len() as u32;
    let mut output = 0_u32;
    let handle = unsafe { GetStdHandle(-11) };
    if handle == -1 {
        return Err(std::io::Error::last_os_error());
    }
    let res = unsafe { WriteConsoleW(handle, msg_ptr, len, &mut output, std::ptr::null()) };
    if res == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn main() {
    let message = "Hello world from syscall!\n";
    syscall(message).unwrap();
    // Hello world from syscall!
}

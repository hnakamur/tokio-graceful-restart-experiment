use std::env;
use std::io;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use std::process::Command;

const LISTEN_FDS: &str = "LISTEN_FDS";

fn main() -> io::Result<()>  {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Listening on {}", listener.local_addr()?);
    let fd = listener.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags == -1 {
        panic!("fcntl F_GETFD failed");
    }
    println!("fd={}, ret={}, fd_cloexec={}, has_f_get_fd={}, new_value={}", fd, flags, libc::FD_CLOEXEC, flags & libc::FD_CLOEXEC, flags & !libc::FD_CLOEXEC);
    let ret = unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) };
    if ret == -1 {
        panic!("fcntl F_SETFD failed");
    }

    let mut it = env::args_os();
    it.next().unwrap();
    let mut cmd = Command::new(it.next().unwrap());
    for arg in it {
        cmd.arg(arg);
    }
    cmd.env(LISTEN_FDS, "1");
    let mut child = cmd.spawn().expect("failed to create child process");
    let ecode = child.wait().expect("failed to get exit status");
    println!("child exit status={}", ecode);
    Ok(())
}

use libc::{kill, SIGTERM};
use std::convert::TryInto;
use std::env;
use std::io;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};
use tokio::stream::StreamExt;

const LISTEN_FDS: &str = "LISTEN_FDS";

fn main() -> io::Result<()> {
    let std_listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Listening on {}", std_listener.local_addr()?);
    let fd = std_listener.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags == -1 {
        panic!("fcntl F_GETFD failed");
    }
    println!(
        "fd={}, ret={}, fd_cloexec={}, has_f_get_fd={}, new_value={}",
        fd,
        flags,
        libc::FD_CLOEXEC,
        flags & libc::FD_CLOEXEC,
        flags & !libc::FD_CLOEXEC
    );
    let ret = unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) };
    if ret == -1 {
        panic!("fcntl F_SETFD failed");
    }

    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .core_threads(2)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut it = env::args_os();
            it.next().unwrap();
            let mut cmd = Command::new(it.next().unwrap());
            for arg in it {
                cmd.arg(arg);
            }
            cmd.env(LISTEN_FDS, "1");
            let child = cmd.spawn().expect("failed to create child process");

            let mut hangup_stream = signal(SignalKind::hangup()).expect("cannot get signal hangup");
            let mut terminate_stream =
                signal(SignalKind::terminate()).expect("cannot get signal terminal");
            let mut user_defined2_stream =
                signal(SignalKind::user_defined2()).expect("cannot get signal user_defined2");
            let mut child_stream = signal(SignalKind::child()).expect("cannot get signal child");

            loop {
                tokio::select! {
                    _ = hangup_stream.next() => {
                        println!("got signal HUP");
                    }
                    _ = terminate_stream.next() => {
                        println!("got signal TERM");
                        unsafe { kill(child.id().try_into().unwrap(), SIGTERM) };
                    }
                    _ = user_defined2_stream.next() => {
                        println!("got signal USR2");
                    }
                    _ = child_stream.next() => {
                        println!("got signal CHLD");
                        break;
                    }
                }
            }
            let status = child.await.expect("failed to await child process");
            println!("child exit status={}", status);
        });
    Ok(())
}

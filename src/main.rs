use nix::unistd::Pid;
use nix::sys::signal::{kill, SIGTERM, Signal};
use std::convert::TryInto;
use std::env;
use std::io;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use tokio::process::{Child, Command};
use tokio::signal::unix::{signal, SignalKind};
use tokio::stream::StreamExt;

const LISTEN_FDS: &str = "LISTEN_FDS";

fn spawn_child() -> io::Result<Child> {
    let mut it = env::args_os();
    it.next().unwrap();
    let mut cmd = Command::new(it.next().unwrap());
    for arg in it {
        cmd.arg(arg);
    }
    cmd.env(LISTEN_FDS, "1");
    cmd.spawn()
}

fn send_signal(child: &Child, sig: Signal) -> nix::Result<()> {
    kill(Pid::from_raw(child.id().try_into().unwrap()), sig)
}

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
            let mut child = spawn_child().expect("failed to create child process");

            let mut hangup_stream = signal(SignalKind::hangup()).expect("cannot get signal hangup");
            let mut terminate_stream =
                signal(SignalKind::terminate()).expect("cannot get signal terminal");
            let mut user_defined2_stream =
                signal(SignalKind::user_defined2()).expect("cannot get signal user_defined2");

            loop {
                tokio::select! {
                    _ = hangup_stream.next() => {
                        println!("got signal HUP");
                    }
                    _ = terminate_stream.next() => {
                        println!("got signal TERM");
                        send_signal(&child, SIGTERM).expect("send SIGTERM to child");
                        break;
                    }
                    _ = user_defined2_stream.next() => {
                        println!("got signal USR2");
                        let new_child = spawn_child().expect("failed to create new child process");
                        send_signal(&child, SIGTERM).expect("send SIGTERM to old child");
                        let status = child.await.expect("child process status");
                        println!("child process exit status={}", status);
                        child = new_child;
                    }
                }
            }
        });
    Ok(())
}

use listenfd::ListenFd;
use std::os::unix::io::AsRawFd;
use tokio::net::TcpListener;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listenfd = ListenFd::from_env();
    let mut listener = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        TcpListener::from_std(listener)?
    } else {
        TcpListener::bind("127.0.0.1:8080").await?
    };
    let fd = listener.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags == -1 {
        panic!("fcntl F_GETFD failed");
    }
    println!("fd={}, ret={}, fd_cloexec={}, has_f_get_fd={}, new_value={}", fd, flags, libc::FD_CLOEXEC, flags & libc::FD_CLOEXEC, flags & !libc::FD_CLOEXEC);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

use actix_web::{web, App, HttpServer, Responder};
use listenfd::ListenFd;
use std::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::stream::StreamExt;

async fn index(info: web::Path<(String, u32)>) -> impl Responder {
    format!("Hello {}! id:{}", info.0, info.1)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    let listener = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        listener
    } else {
        TcpListener::bind("127.0.0.1:8080")?
    };

    let server =
        HttpServer::new(|| App::new().service(web::resource("/{name}/{id}/index.html").to(index)))
            .listen(listener)?
            .run();

    let srv = server.clone();
    tokio::spawn(async move {
        let mut terminate_stream =
            signal(SignalKind::terminate()).expect("cannot get signal terminal");
        loop {
            tokio::select! {
                _ = terminate_stream.next() => {
                    println!("http-server got signal TERM, start graceful shutdown");
                    srv.stop(true).await;
                    break;
                },
            }
        }
    });

    server.await
}

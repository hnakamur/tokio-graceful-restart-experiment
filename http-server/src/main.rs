use actix_web::{web, App, Responder, HttpServer};
use listenfd::ListenFd;
use std::net::TcpListener;

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

    HttpServer::new(|| App::new().service(
        web::resource("/{name}/{id}/index.html").to(index))
    )
        .listen(listener)?
        .run()
        .await
}

use std::io::{Read, Write};

use actix_web::{get, web, Error, HttpResponse};
use tracing::trace;

use crate::Pool;

#[get("/hello/{name}")]
pub async fn greet(
    name: web::Path<String>,
    tcp_pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let mut tcp = tcp_pool.get().await.unwrap();
    tcp.write_all(b"Hello, This is actix-web server!")?;
    // 读取响应
    let mut buffer = [0; 1024];
    tcp.read(&mut buffer)?;

    trace!("Received: {}", String::from_utf8_lossy(&buffer));
    Ok(HttpResponse::Ok().body(format!("Hello {name}!")))
}

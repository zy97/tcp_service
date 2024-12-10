mod server_router;
mod tcp_manager;
use actix_web::{middleware, web, App, HttpServer};
use deadpool::managed::{self};
use dotenv::dotenv;
use server_router::greet;
use std::{env, str};

use tcp_manager::ModbusContext;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

type Pool = managed::Pool<ModbusContext>;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mgr = ModbusContext {
        addr: get_env("ips"),
    };
    let tcp_pool = Pool::builder(mgr).max_size(1).build().unwrap();
    let server_url = get_env("server");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tcp_pool.clone()))
            .wrap(middleware::Logger::default())
            .service(greet)
    })
    .bind(&server_url)?
    .run();
    info!("Server running! Access the index page here: http://{server_url}/",);
    server.await
}
fn get_env(key: impl AsRef<str>) -> String {
    match env::var(key.as_ref()) {
        Ok(s) if !s.is_empty() => {
            info!("found key {}:{}", key.as_ref(), s);
            s
        }
        _ => panic!("no found"),
    }
}

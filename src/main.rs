use std::sync::Mutex;
use actix_web::{App, HttpServer, web::{self}};
use crate::{routes::user::{sign_in, sign_up}, types::auth::User};

pub mod types;
pub mod routes;

struct AppState {
    users: Mutex<Vec<User>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState {
        users: Mutex::new(vec![])
    });
    HttpServer::new(move || {
        App::new()
        .app_data(app_data.clone())
        .service(sign_up)
        .service(sign_in)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
pub(crate) mod handler;
pub(crate) mod models;
pub(crate) mod responses;

use crate::{handler::web_service_config, models::AppState};
use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
    let todo_db = AppState::init();
    let app_data = web::Data::new(todo_db);

    println!("🚀 Server started successfully");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3000/")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .configure(web_service_config)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

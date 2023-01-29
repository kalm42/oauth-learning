mod api;
use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{get, Responder, HttpServer, HttpResponse, App};
use actix_web::cookie::Key;
use dotenv::dotenv;
use api::oauth_github;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

fn get_secret_key() -> Key {
    Key::generate()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let secret_key = get_secret_key();
    let redis_connection_string = "redis://127.0.0.1:6379";
    let store = match RedisSessionStore::new(redis_connection_string).await {
        Ok(store) => store,
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::new(store.clone(), secret_key.clone())
            )
            .service(index)
            .service(oauth_github::login)
            .service(oauth_github::callback)
    }).bind(("127.0.0.1", 5000)) {
        Ok(server) => server.run().await,
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

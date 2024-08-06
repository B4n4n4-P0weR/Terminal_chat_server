use actix_web::{web, App, HttpServer, Responder};
use tokio_postgres::NoTls;
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify};

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

async fn register_user(user: web::Json<User>, db_pool: web::Data<tokio_postgres::Client>) -> impl Responder {
    let hashed_password = hash(&user.password, 4).unwrap();
    let result = db_pool
        .execute(
            "INSERT INTO users (username, password) VALUES ($1, $2)",
            &[&user.username, &hashed_password],
        )
        .await;

    match result {
        Ok(_) => format!("User {} registered successfully!", user.username),
        Err(err) => format!("Failed to register user: {}", err),
    }
}

async fn login_user(user: web::Json<User>, db_pool: web::Data<tokio_postgres::Client>) -> impl Responder {
    let row = db_pool
        .query_one("SELECT password FROM users WHERE username=$1", &[&user.username])
        .await
        .unwrap();
    let stored_password: &str = row.get(0);

    if verify(&user.password, stored_password).unwrap() {
        format!("User {} logged in successfully!", user.username)
    } else {
        format!("Invalid credentials!")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (db_client, db_connection) =
        tokio_postgres::connect("host=localhost user=postgres password=yourpassword dbname=chat_db", NoTls)
            .await
            .unwrap();

    // Spawning a task to manage the connection
    tokio::spawn(async move {
        if let Err(e) = db_connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let db_pool = web::Data::new(db_client);

    HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login_user))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

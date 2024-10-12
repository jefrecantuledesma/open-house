use actix_files::NamedFile;
use actix_web::{web, App, HttpServer};
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;
mod login;
mod register;
mod user;

// Serve the index.html file
async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

// Serve user pages
async fn user_page(
    username: web::Path<String>,
    req: actix_web::HttpRequest,
) -> actix_web::Result<NamedFile> {
    // Validate username
    let username_str = username.into_inner();
    if !user::is_valid_username(&username_str) {
        return Ok(NamedFile::open("./static/404.html")?);
    }

    let logged_in_username = req
        .cookie("username")
        .map(|cookie| cookie.value().to_string());
    let user_page_path = format!("./user_pages/{}/my_page.html", username_str);

    if Some(username_str.clone()) == logged_in_username {
        // The logged-in user is accessing their own page
        Ok(NamedFile::open(user_page_path)?)
    } else {
        // A different user is accessing the page (view only)
        let view_only_path = format!("./user_pages/{}/view_only.html", username_str);
        if Path::new(&view_only_path).exists() {
            Ok(NamedFile::open(view_only_path)?)
        } else {
            Ok(NamedFile::open(user_page_path)?)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create a connection pool
    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:users.db")
        .await
        .expect("Failed to create pool.");

    // Create the users table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL,
            has_logged_in BOOLEAN NOT NULL DEFAULT 0
        );",
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create users table");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(actix_files::Files::new("/user_pages", "./user_pages").show_files_listing())
            .route("/", web::get().to(index))
            .route("/register", web::post().to(register::register_user))
            .route("/login", web::post().to(login::login_user))
            .route("/user_pages/{username}", web::get().to(user_page))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

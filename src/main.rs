use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::path::Path;
mod customize;
mod friends;
mod invite;
mod login;
mod register;
mod user;

// Serve the index.html file
async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

// Serve user pages
async fn user_page(
    path: web::Path<(String, String)>,
    req: HttpRequest,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let (username, filename) = path.into_inner();

    // Prevent directory traversal and invalid paths
    if filename.contains("..") || filename.starts_with("/") || filename.starts_with("\\") {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // Validate username and prevent directory traversal
    if !user::is_valid_username(&username) || filename.contains("..") {
        return Ok(HttpResponse::NotFound().finish());
    }

    let allowed_extensions = [
        "html", "css", "js", "jpg", "jpeg", "png", "gif", "svg", "webp", "bmp", "mp4", "webm",
        "ogg", "mp3", "wav", "ogg", "flac",
    ];

    let file_extension = filename.split('.').last().unwrap_or("");
    if !allowed_extensions.contains(&file_extension) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let logged_in_username = req
        .cookie("username")
        .map(|cookie| cookie.value().to_string());

    let is_css_or_js = filename.ends_with(".css") || filename.ends_with(".js");

    if Some(username.clone()) == logged_in_username || is_css_or_js {
        // Allow access to own files or CSS/JS files
        let user_file_path = format!("./user_pages/{}/{}", username, filename);

        if Path::new(&user_file_path).exists() {
            // Serve the file with correct content type
            Ok(NamedFile::open(user_file_path)?.into_response(&req))
        } else {
            // File not found
            Ok(HttpResponse::NotFound().finish())
        }
    } else {
        // Check if the logged-in user is a friend of the requested user
        if let Some(logged_in_user) = logged_in_username {
            let is_friend = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM friends WHERE (user1 = ? AND user2 = ?) OR (user1 = ? AND user2 = ?)",
            )
            .bind(&logged_in_user)
            .bind(&username)
            .bind(&username)
            .bind(&logged_in_user)
            .fetch_one(pool.get_ref())
            .await
            .unwrap_or(0) > 0;

            if is_friend {
                // Allow access to friend's pages
                let user_file_path = format!("./user_pages/{}/{}", username, filename);

                if Path::new(&user_file_path).exists() {
                    Ok(NamedFile::open(user_file_path)?.into_response(&req))
                } else {
                    Ok(HttpResponse::NotFound().finish())
                }
            } else {
                Ok(HttpResponse::Forbidden().finish())
            }
        } else {
            Ok(HttpResponse::Unauthorized().finish())
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

    // Create the invite_tokens table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS invite_tokens (
        token TEXT PRIMARY KEY,
        username TEXT NOT NULL,
        FOREIGN KEY(username) REFERENCES users(username)
    );",
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create invite_tokens table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS friends (
        user1 TEXT NOT NULL,
        user2 TEXT NOT NULL,
        PRIMARY KEY (user1, user2),
        FOREIGN KEY(user1) REFERENCES users(username),
        FOREIGN KEY(user2) REFERENCES users(username)
    );",
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create friends table");

    HttpServer::new(move || {
        let db_pool_clone = db_pool.clone();
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            // Remove or secure the following line to prevent direct access to user_pages
            // .service(actix_files::Files::new("/user_pages", "./user_pages").show_files_listing())
            .route("/", web::get().to(index))
            .route("/register", web::post().to(register::register_user))
            .route("/login", web::post().to(login::login_user))
            .route(
                "/user_pages/{username}/{filename:.*}",
                web::get().to({
                    let db_pool_inner = db_pool_clone.clone(); // Clone inside the closure
                    move |path, req| {
                        user_page(path, req, web::Data::new(db_pool_inner.clone()))
                        // Clone again here
                    }
                }),
            )
            .route("/save_changes", web::post().to(customize::save_changes))
            .route("/generate_invite", web::get().to(invite::generate_invite))
            .route("/invite/{token}", web::get().to(invite::handle_invite))
            .route("/upload_gallery", web::post().to(customize::upload_gallery))
            .route("/get_galleries", web::get().to(customize::get_galleries))
            .route("/get_friends", web::get().to(friends::get_friends))
            .route("/add_friend", web::post().to(friends::add_friend))
            .route(
                "/upload_text_post",
                web::post().to(customize::upload_text_post),
            )
            .route("/get_text_posts", web::get().to(customize::get_text_posts))
            .route("/upload_film", web::post().to(customize::upload_film))
            .route("/get_films", web::get().to(customize::get_films))
            .route("/upload_audio", web::post().to(customize::upload_audio))
            .route("/get_audios", web::get().to(customize::get_audios))
            // Inside your HttpServer configuration
            .route(
                "/get_all_content",
                web::get().to(customize::get_all_content),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

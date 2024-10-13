use actix_files::NamedFile;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct InviteLinkResponse {
    pub invite_link: String,
}

pub async fn generate_invite(req: HttpRequest, pool: web::Data<SqlitePool>) -> HttpResponse {
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    // Generate a unique token
    let token = Uuid::new_v4().to_string();

    // Insert token into invite_tokens table
    if let Err(e) = sqlx::query("INSERT INTO invite_tokens (token, username) VALUES (?, ?)")
        .bind(&token)
        .bind(&username)
        .execute(pool.get_ref())
        .await
    {
        return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
    }

    // Return the invite code
    HttpResponse::Ok().json(json!({ "invite_code": token }))
}
pub async fn handle_invite(
    token: web::Path<String>,
    db_pool: web::Data<sqlx::SqlitePool>,
) -> actix_web::Result<NamedFile> {
    let token_str = token.into_inner();

    // Fetch the username associated with the token
    let result = sqlx::query("SELECT username FROM invite_tokens WHERE token = ?")
        .bind(&token_str)
        .fetch_one(db_pool.get_ref())
        .await;

    match result {
        Ok(row) => {
            let username: String = row.get("username");

            // Delete the token to make it one-time use
            let _ = sqlx::query("DELETE FROM invite_tokens WHERE token = ?")
                .bind(&token_str)
                .execute(db_pool.get_ref())
                .await;

            // Serve the user's page
            let user_page_path = format!("./user_pages/{}/my_page.html", username);
            Ok(NamedFile::open(user_page_path)?)
        }
        Err(sqlx::Error::RowNotFound) => {
            // Token not found or already used
            Ok(NamedFile::open("./static/404.html")?)
        }
        Err(err) => {
            eprintln!("Database query error: {}", err);
            Ok(NamedFile::open("./static/500.html")?)
        }
    }
}

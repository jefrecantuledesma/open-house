use crate::user;
use actix_web::{web, HttpResponse, Responder};
use base64::encode;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
}

pub async fn register_user(
    db_pool: web::Data<sqlx::SqlitePool>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    // Validate username
    if !user::is_valid_username(&req.username) {
        return HttpResponse::BadRequest()
            .body("Invalid username. Allowed characters: a-z, A-Z, 0-9, -, _");
    }

    // Validate password length
    if req.password.len() < 8 {
        return HttpResponse::BadRequest().body("Password must be at least 8 characters long.");
    }

    // Check if passwords match
    if req.password != req.confirm_password {
        return HttpResponse::BadRequest().body("Passwords do not match.");
    }

    // Check if username already exists
    let existing_user = sqlx::query_as::<_, user::User>(
        "SELECT username, password_hash, has_logged_in FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_optional(db_pool.get_ref())
    .await;

    match existing_user {
        Ok(Some(_)) => {
            // Username already exists
            return HttpResponse::BadRequest().body("Username already exists.");
        }
        Ok(None) => {
            // Username is available
        }
        Err(err) => {
            eprintln!("Database query error: {}", err);
            return HttpResponse::InternalServerError().body("Error during registration");
        }
    }

    // Hash the password
    let mut hasher = Sha256::new();
    hasher.update(&req.password);
    let password_hash = encode(hasher.finalize());

    // Insert into the database
    let result =
        sqlx::query("INSERT INTO users (username, password_hash, has_logged_in) VALUES (?, ?, ?)")
            .bind(&req.username)
            .bind(&password_hash)
            .bind(false)
            .execute(db_pool.get_ref())
            .await;

    match result {
        Ok(_) => {
            // Return success message
            HttpResponse::Ok().body("Registration successful")
        }
        Err(err) => {
            eprintln!("Database insert error: {}", err);
            HttpResponse::InternalServerError().body("Error registering user")
        }
    }
}

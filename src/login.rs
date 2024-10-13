use crate::user;
use actix_web::cookie::{Cookie, CookieBuilder, SameSite}; // Added import
use actix_web::{web, HttpResponse, Responder};
use base64::encode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login_user(
    db_pool: web::Data<sqlx::SqlitePool>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    // Validate username
    if !user::is_valid_username(&req.username) {
        return HttpResponse::BadRequest().body("Invalid username.");
    }

    // Fetch the user from the database
    let result = sqlx::query_as::<_, user::User>(
        "SELECT username, password_hash, has_logged_in FROM users WHERE username = ?",
    )
    .bind(&req.username)
    .fetch_one(db_pool.get_ref())
    .await;

    match result {
        Ok(mut user) => {
            // Hash the provided password
            let mut hasher = Sha256::new();
            hasher.update(&req.password);
            let password_hash = encode(hasher.finalize());

            if password_hash == user.password_hash {
                if !user.has_logged_in {
                    // Create user page directory if it doesn't exist
                    let user_page_path = format!("./user_pages/{}/", user.username);
                    if !Path::new(&user_page_path).exists() {
                        if let Err(e) = fs::create_dir_all(&user_page_path) {
                            eprintln!("Failed to create user page directory: {}", e);
                            return HttpResponse::InternalServerError()
                                .body("Error creating user directory.");
                        }
                        if let Err(e) = fs::copy(
                            "./user_pages/default_page.html",
                            format!("{}my_page.html", &user_page_path),
                        ) {
                            eprintln!("Failed to copy default user HTML: {}", e);
                            return HttpResponse::InternalServerError()
                                .body("Error copying default user HTML.");
                        };
                        if let Err(e) = fs::copy(
                            "./user_pages/default_styles.css",
                            format!("{}my_styles.css", &user_page_path),
                        ) {
                            eprintln!("Failed to copy default user CSS: {}", e);
                            return HttpResponse::InternalServerError()
                                .body("Error copying default user CSS.");
                        };
                        if let Err(e) = fs::copy(
                            "./user_pages/default_scripts.js",
                            format!("{}my_scripts.js", &user_page_path),
                        ) {
                            eprintln!("Failed to copy default user JavaScript: {}", e);
                            return HttpResponse::InternalServerError()
                                .body("Error copying default user JavaScript.");
                        };
                    }

                    let user_html_path = format!("{}my_page.html", &user_page_path);
                    if let Err(e) = modify_html_for_user(&user_html_path, &user.username) {
                        eprintln!("Failed to modify HTML for user: {}", e);
                        return HttpResponse::InternalServerError()
                            .body("Error modifying HTML for user.");
                    }

                    // Update has_logged_in to true
                    let update_result =
                        sqlx::query("UPDATE users SET has_logged_in = ? WHERE username = ?")
                            .bind(true)
                            .bind(&user.username)
                            .execute(db_pool.get_ref())
                            .await;

                    if let Err(err) = update_result {
                        eprintln!("Database update error: {}", err);
                        return HttpResponse::InternalServerError()
                            .body("Error updating user login status");
                    }
                }

                // Set session cookie with attributes
                let cookie = CookieBuilder::new("username", user.username.clone())
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .secure(false) // Set to true if using HTTPS
                    .finish();

                HttpResponse::Ok()
                    .cookie(cookie)
                    .body(format!("/user_pages/{}/my_page.html", user.username))
            } else {
                // Invalid password
                HttpResponse::Unauthorized().body("Invalid password")
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            // User not found
            HttpResponse::BadRequest().body("User not found")
        }
        Err(err) => {
            eprintln!("Database query error: {}", err);
            HttpResponse::InternalServerError().body("Error during login")
        }
    }
}

fn modify_html_for_user(html_path: &str, username: &str) -> std::io::Result<()> {
    // Read the contents of the HTML file
    let mut html_content = fs::read_to_string(html_path)?;

    html_content = html_content.replace("{{username}}", username);

    // Create absolute paths for CSS and JS files
    let css_link = format!(
        r#"<link rel="stylesheet" href="/user_pages/{}/my_styles.css">"#,
        username
    );
    let script_tag = format!(
        r#"<script defer src="/user_pages/{}/my_scripts.js"></script>"#,
        username
    );

    // Insert the CSS link and script tag into the <head> section of the HTML
    if let Some(head_index) = html_content.find("</head>") {
        html_content.insert_str(head_index, &format!("{}\n{}\n", css_link, script_tag));
    } else {
        // If </head> not found, add at the beginning
        html_content = format!("{}\n{}\n{}", css_link, script_tag, html_content);
    }

    // Write the modified content back to the file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(html_path)?;
    file.write_all(html_content.as_bytes())?;

    Ok(())
}

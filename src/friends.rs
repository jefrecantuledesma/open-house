use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};
use url::Url;

#[derive(Deserialize)]
pub struct AddFriendData {
    pub invite_code: String,
}

pub async fn add_friend(
    data: web::Json<AddFriendData>,
    req: HttpRequest,
    pool: web::Data<SqlitePool>,
) -> HttpResponse {
    // Extract the username from the cookie (user who is adding a friend)
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized().body("User not authenticated");
        }
    };

    println!("add_friend: username from cookie: {}", username);

    // Get the invite code directly
    let token = &data.invite_code;

    // Retrieve the inviting user from the invite_tokens table
    match sqlx::query("SELECT username FROM invite_tokens WHERE token = ?")
        .bind(token)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => {
            let inviting_user: String = row.get("username");

            println!(
                "add_friend: inviting_user from invite code: {}",
                inviting_user
            );

            // Prevent users from adding themselves as friends
            if username == inviting_user {
                return HttpResponse::BadRequest().body("Cannot add yourself as a friend.");
            }

            // Begin a transaction
            let mut tx = match pool.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .body(format!("Database error: {}", e));
                }
            };

            // Insert the friendship into the friends table (both directions)
            if let Err(e) =
                sqlx::query("INSERT OR IGNORE INTO friends (user1, user2) VALUES (?, ?)")
                    .bind(&username)
                    .bind(&inviting_user)
                    .execute(&mut tx)
                    .await
            {
                return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
            }

            if let Err(e) =
                sqlx::query("INSERT OR IGNORE INTO friends (user1, user2) VALUES (?, ?)")
                    .bind(&inviting_user)
                    .bind(&username)
                    .execute(&mut tx)
                    .await
            {
                return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
            }

            // Delete the invite token after use
            if let Err(e) = sqlx::query("DELETE FROM invite_tokens WHERE token = ?")
                .bind(token)
                .execute(&mut tx)
                .await
            {
                return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
            }

            // Commit the transaction
            if let Err(e) = tx.commit().await {
                return HttpResponse::InternalServerError().body(format!("Database error: {}", e));
            }

            HttpResponse::Ok().body("Friend added successfully.")
        }
        Err(_) => HttpResponse::BadRequest().body("Invalid or expired invite code."),
    }
}

pub async fn get_friends(req: HttpRequest, pool: web::Data<SqlitePool>) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized().body("User not authenticated");
        }
    };

    // Retrieve friends from the database
    match sqlx::query(
        "SELECT user2 as friend FROM friends WHERE user1 = ? AND user2 != ?
         UNION
         SELECT user1 as friend FROM friends WHERE user2 = ? AND user1 != ?",
    )
    .bind(&username)
    .bind(&username)
    .bind(&username)
    .bind(&username)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let friends: Vec<String> = rows.into_iter().map(|row| row.get("friend")).collect();
            HttpResponse::Ok().json(friends)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

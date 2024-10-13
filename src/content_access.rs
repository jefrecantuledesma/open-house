// ContentAccess.rs
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Serialize)]
struct Photo {
    filename: String,
    url: String, // URL to access the photo
}

pub async fn get_friend_photos(
    pool: web::Data<SqlitePool>,
    req: HttpRequest,
    path: web::Path<String>, // friend's username
) -> impl Responder {
    let requester = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    let friend_username = path.into_inner();

    // Prevent users from accessing their own photos via this endpoint
    if requester == friend_username {
        return HttpResponse::BadRequest().body("Cannot access your own photos via this endpoint.");
    }

    // Check if requester and friend are friends
    let friendship = sqlx::query("SELECT 1 FROM friends WHERE user1 = ? AND user2 = ? LIMIT 1")
        .bind(&requester)
        .bind(&friend_username)
        .fetch_optional(pool.get_ref())
        .await;

    match friendship {
        Ok(Some(_)) => {
            // They are friends, retrieve photos
            let photos = sqlx::query("SELECT filename FROM photos WHERE username = ?")
                .bind(&friend_username)
                .fetch_all(pool.get_ref())
                .await;

            match photos {
                Ok(rows) => {
                    let photos: Vec<Photo> = rows
                        .into_iter()
                        .map(|row| {
                            let filename: String = row.get("filename");
                            Photo {
                                filename: filename.clone(),
                                url: format!("/user_pages/{}/photos/{}", friend_username, filename),
                            }
                        })
                        .collect();

                    HttpResponse::Ok().json(photos)
                }
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Database error: {}", e))
                }
            }
        }
        Ok(None) => {
            // Not friends
            HttpResponse::Forbidden().body("You are not friends with this user.")
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

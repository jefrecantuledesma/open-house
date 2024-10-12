use regex::Regex;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub has_logged_in: bool,
}

pub fn is_valid_username(username: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    re.is_match(username)
}

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use kuchiki::traits::*;
use kuchiki::NodeRef;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ContentItem {
    Gallery(Gallery),
    TextPost(TextPost),
    Film(Film),
    Audio(Audio),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveChangesData {
    pub exhibit_title: String,
    pub main_title: String,
}

#[derive(Serialize, Deserialize)]
struct Gallery {
    title: String,
    images: Vec<String>,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct Film {
    title: String,
    video_path: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct Audio {
    title: String,
    audio_path: String,
    timestamp: String,
}

#[derive(Deserialize)]
pub struct TextPostInput {
    pub title: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct TextPost {
    pub title: String,
    pub content: String,
    pub timestamp: String,
}

pub async fn save_changes(data: web::Json<SaveChangesData>, req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized().body("User not authenticated");
        }
    };

    // Validate the input to prevent injection attacks
    if data.exhibit_title.contains('<')
        || data.exhibit_title.contains('>')
        || data.main_title.contains('<')
        || data.main_title.contains('>')
    {
        return HttpResponse::BadRequest().body("Invalid input detected");
    }

    // Path to the user's HTML file
    let user_page_path = format!("./user_pages/{}/my_page.html", username);

    // Check if the user's page exists
    if !Path::new(&user_page_path).exists() {
        return HttpResponse::NotFound().body("User page not found");
    }

    // Read the current content of the HTML file
    let html_content = match fs::read_to_string(&user_page_path) {
        Ok(content) => content,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to read user page");
        }
    };

    // Replace the titles in the HTML content
    let updated_html =
        match update_html_titles(&html_content, &data.exhibit_title, &data.main_title) {
            Ok(html) => html,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to update HTML: {}", e));
            }
        };

    // Write the updated content back to the file
    match fs::write(&user_page_path, updated_html) {
        Ok(_) => HttpResponse::Ok().body("Changes saved successfully"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to write to user page"),
    }
}

// Function to update the titles in the HTML content
fn update_html_titles(
    html_content: &str,
    new_exhibit_title: &str,
    new_main_title: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Parse the HTML content
    let document = kuchiki::parse_html().one(html_content);

    // Update the <title> element
    if let Ok(title_element) = document.select_first("title") {
        let node = title_element.as_node();
        // Collect the children into a vector to avoid borrowing issues
        let children: Vec<_> = node.children().collect();
        // Detach all children
        for child in children {
            child.detach();
        }
        // Append the new text
        node.append(NodeRef::new_text(new_exhibit_title));
    }

    // Update the <header><h1> element
    if let Ok(header_h1_element) = document.select_first("header h1") {
        let node = header_h1_element.as_node();
        let children: Vec<_> = node.children().collect();
        for child in children {
            child.detach();
        }
        node.append(NodeRef::new_text(new_exhibit_title));
    }

    // Update the <main><h2> element
    if let Ok(main_h2_element) = document.select_first("main h2") {
        let node = main_h2_element.as_node();
        let children: Vec<_> = node.children().collect();
        for child in children {
            child.detach();
        }
        node.append(NodeRef::new_text(new_main_title));
    }

    // Serialize the updated HTML content back to a string
    let mut updated_html = Vec::new();
    document.serialize(&mut updated_html)?;

    Ok(String::from_utf8(updated_html)?)
}

pub async fn upload_audio(mut payload: Multipart, req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    let mut audio_title = String::new();
    let mut audio_path = String::new();
    let mut audio_uploaded = false;

    // Get the current timestamp for unique naming
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Path to the audios folder
    let audios_folder = format!("./user_pages/{}/audios", username);
    fs::create_dir_all(&audios_folder).unwrap();

    // Process the multipart form data
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(_) => continue,
        };

        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.unwrap().get_name() {
            if name == "audioTitle" {
                // Read the audio title
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(_) => continue,
                    };
                    data.extend_from_slice(&chunk);
                }
                audio_title = String::from_utf8(data).unwrap_or_default();
            } else if name == "audio" {
                // Ensure only one audio file is uploaded
                if audio_uploaded {
                    return HttpResponse::BadRequest().body("Only one audio file is allowed.");
                }

                // Get the filename
                let filename = content_disposition
                    .unwrap()
                    .get_filename()
                    .map(|f| sanitize_filename::sanitize(f))
                    .unwrap_or_else(|| format!("audio_{}.mp3", timestamp));

                // Check file size limit (50MB)
                let mut data = web::BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(_) => continue,
                    };
                    if (data.len() + chunk.len()) > 50 * 1024 * 1024 {
                        return HttpResponse::BadRequest()
                            .body("File size too big (must be under 50MB).");
                    }
                    data.extend_from_slice(&chunk);
                }

                // Save the file
                let file_path = format!("{}/{}", audios_folder, filename);
                let mut f = web::block(move || std::fs::File::create(&file_path))
                    .await
                    .unwrap()
                    .unwrap();
                if let Err(_) = f.write_all(&data) {
                    return HttpResponse::InternalServerError().body("Error saving audio file.");
                }

                // Set the audio path
                audio_path = format!("/user_pages/{}/audios/{}", username, filename);
                audio_uploaded = true;
            }
        }
    }

    // Validate that an audio file was uploaded
    if !audio_uploaded {
        return HttpResponse::BadRequest().body("Please upload an audio file.");
    }

    // Save audio metadata (could be saved in a database; for now, we'll save in a JSON file)
    let audio_metadata = Audio {
        title: audio_title.clone(),
        audio_path: audio_path.clone(),
        timestamp: timestamp.clone(),
    };

    let metadata_filename = format!("{}.json", timestamp);
    let metadata_path = format!("{}/{}", audios_folder, metadata_filename);
    if let Err(_) = fs::write(
        &metadata_path,
        serde_json::to_string(&audio_metadata).unwrap(),
    ) {
        return HttpResponse::InternalServerError().body("Error saving audio metadata.");
    }

    HttpResponse::Ok().body("Audio uploaded successfully.")
}

pub async fn get_audios(req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    let audios_folder = format!("./user_pages/{}/audios", username);
    let mut audios = Vec::new();

    if let Ok(entries) = fs::read_dir(&audios_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(audio) = serde_json::from_str::<Audio>(&data) {
                        audios.push(audio);
                    }
                }
            }
        }
    }

    // Sort audios by title or any other criteria if needed
    // For now, we'll leave them in the order they were read
    HttpResponse::Ok().json(audios)
}

pub async fn upload_film(mut payload: Multipart, req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    let mut film_title = String::new();
    let mut video_path = String::new();
    let mut video_uploaded = false;

    // Get the current timestamp for unique naming
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Path to the films folder
    let films_folder = format!("./user_pages/{}/films", username);
    fs::create_dir_all(&films_folder).unwrap();

    // Process the multipart form data
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(_) => continue,
        };

        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.unwrap().get_name() {
            if name == "filmTitle" {
                // Read the film title
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(_) => continue,
                    };
                    data.extend_from_slice(&chunk);
                }
                film_title = String::from_utf8(data).unwrap_or_default();
            } else if name == "video" {
                // Ensure only one video is uploaded
                if video_uploaded {
                    return HttpResponse::BadRequest().body("Only one video is allowed.");
                }

                // Get the filename
                let filename = content_disposition
                    .unwrap()
                    .get_filename()
                    .map(|f| sanitize_filename::sanitize(f))
                    .unwrap_or_else(|| format!("video_{}.mp4", timestamp));

                // Check file size limit (200MB)
                let mut data = web::BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(_) => continue,
                    };
                    if (data.len() + chunk.len()) > 200 * 1024 * 1024 {
                        return HttpResponse::BadRequest()
                            .body("File size too big (must be under 200MB).");
                    }
                    data.extend_from_slice(&chunk);
                }

                // Save the file
                let file_path = format!("{}/{}", films_folder, filename);
                let mut f = web::block(move || std::fs::File::create(&file_path))
                    .await
                    .unwrap()
                    .unwrap();
                if let Err(_) = f.write_all(&data) {
                    return HttpResponse::InternalServerError().body("Error saving video file.");
                }

                // Set the video path
                video_path = format!("/user_pages/{}/films/{}", username, filename);
                video_uploaded = true;
            }
        }
    }

    // Validate that a video was uploaded
    if !video_uploaded {
        return HttpResponse::BadRequest().body("Please upload a video file.");
    }

    // Save film metadata (could be saved in a database; for now, we'll save in a JSON file)
    let film_metadata = Film {
        title: film_title.clone(),
        video_path: video_path.clone(),
        timestamp: timestamp.clone(),
    };

    let metadata_filename = format!("{}.json", timestamp);
    let metadata_path = format!("{}/{}", films_folder, metadata_filename);
    if let Err(_) = fs::write(
        &metadata_path,
        serde_json::to_string(&film_metadata).unwrap(),
    ) {
        return HttpResponse::InternalServerError().body("Error saving film metadata.");
    }

    HttpResponse::Ok().body("Film uploaded successfully.")
}

pub async fn get_films(req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    let films_folder = format!("./user_pages/{}/films", username);
    let mut films = Vec::new();

    if let Ok(entries) = fs::read_dir(&films_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(film) = serde_json::from_str::<Film>(&data) {
                        films.push(film);
                    }
                }
            }
        }
    }

    // Sort films by title or any other criteria if needed
    // For now, we'll leave them in the order they were read
    HttpResponse::Ok().json(films)
}

pub async fn upload_gallery(mut payload: Multipart, req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized().body("User not authenticated");
        }
    };

    // Create a vector to hold the image paths
    let mut image_paths = Vec::new();
    let mut gallery_title = String::new();
    let mut image_count = 0;

    // Get the current timestamp for the gallery folder
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Path to the gallery folder
    let gallery_folder = format!("./user_pages/{}/gallery/{}", username, timestamp);
    fs::create_dir_all(&gallery_folder).unwrap();

    // Process the multipart form data
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(_) => continue,
        };

        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.unwrap().get_name() {
            if name == "galleryTitle" {
                // Read the gallery title
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(_) => continue,
                    };
                    data.extend_from_slice(&chunk);
                }
                gallery_title = String::from_utf8(data).unwrap_or_default();
            } else if name == "images" {
                // Limit to 20 images
                if image_count >= 20 {
                    return HttpResponse::BadRequest().body("Maximum of 20 images allowed.");
                }

                // Get the filename
                let filename = content_disposition
                    .unwrap()
                    .get_filename()
                    .map(|f| sanitize_filename::sanitize(f))
                    .unwrap_or_else(|| format!("image_{}.png", image_count));

                // Check file size limit (10MB)
                let mut data = web::BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(_) => continue,
                    };
                    if (data.len() + chunk.len()) > 10 * 1024 * 1024 {
                        return HttpResponse::BadRequest()
                            .body("File size too big (must be under 10MB).");
                    }
                    data.extend_from_slice(&chunk);
                }

                // Save the file
                let file_path = format!("{}/{}", gallery_folder, filename);
                let mut f = web::block(move || std::fs::File::create(file_path))
                    .await
                    .unwrap()
                    .unwrap();
                if let Err(_) = f.write_all(&data) {
                    return HttpResponse::InternalServerError().body("Error saving file.");
                }

                // Add the image path to the list
                image_paths.push(format!(
                    "/user_pages/{}/gallery/{}/{}",
                    username, timestamp, filename
                ));
                image_count += 1;
            }
        }
    }

    // Validate that at least one image was uploaded
    if image_paths.is_empty() {
        return HttpResponse::BadRequest().body("Please upload at least one image.");
    }

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Save gallery metadata (could be saved in a database; for now, we'll save in a JSON file)
    let gallery_metadata = Gallery {
        title: gallery_title.clone(),
        images: image_paths.clone(),
        timestamp: timestamp.clone(),
    };

    let metadata_path = format!("{}/metadata.json", gallery_folder);
    if let Err(_) = fs::write(
        &metadata_path,
        serde_json::to_string(&gallery_metadata).unwrap(),
    ) {
        return HttpResponse::InternalServerError().body("Error saving gallery metadata.");
    }

    HttpResponse::Ok().body("Gallery uploaded successfully.")
}

pub async fn get_galleries(req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized().body("User not authenticated");
        }
    };

    let galleries_folder = format!("./user_pages/{}/gallery", username);
    let mut galleries = Vec::new();

    if let Ok(entries) = fs::read_dir(&galleries_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let metadata_path = path.join("metadata.json");
                if metadata_path.exists() {
                    if let Ok(data) = fs::read_to_string(&metadata_path) {
                        if let Ok(gallery) = serde_json::from_str::<Gallery>(&data) {
                            galleries.push(gallery);
                        }
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(galleries)
}

pub async fn upload_text_post(data: web::Json<TextPostInput>, req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    // Validate input to prevent injection attacks
    if data.title.contains('<')
        || data.title.contains('>')
        || data.content.contains('<')
        || data.content.contains('>')
    {
        return HttpResponse::BadRequest().body("Invalid input detected");
    }

    // Get the current timestamp for unique file naming
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Path to the text posts folder
    let text_posts_folder = format!("./user_pages/{}/text_posts", username);
    fs::create_dir_all(&text_posts_folder).unwrap();

    // Create a TextPost object with the current timestamp
    let text_post_input = TextPostInput {
        title: data.title.clone(),
        content: data.content.clone(),
    };

    let text_post = TextPost {
        title: text_post_input.title,
        content: text_post_input.content,
        timestamp: timestamp.clone(),
    };

    // Save the text post as a JSON file
    let file_path = format!("{}/{}.json", text_posts_folder, timestamp);
    match fs::write(&file_path, serde_json::to_string(&text_post).unwrap()) {
        Ok(_) => HttpResponse::Ok().body("Text post uploaded successfully."),
        Err(_) => HttpResponse::InternalServerError().body("Error saving text post."),
    }
}

pub async fn get_text_posts(req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => return HttpResponse::Unauthorized().body("User not authenticated"),
    };

    let text_posts_folder = format!("./user_pages/{}/text_posts", username);
    let mut text_posts = Vec::new();

    if let Ok(entries) = fs::read_dir(&text_posts_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(text_post) = serde_json::from_str::<TextPost>(&data) {
                        text_posts.push(text_post);
                    }
                }
            }
        }
    }

    // Sort text posts by timestamp in descending order (newest first)
    text_posts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    HttpResponse::Ok().json(text_posts)
}

pub async fn get_all_content(req: HttpRequest) -> HttpResponse {
    // Extract the username from the cookie
    let username = match req.cookie("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return HttpResponse::Unauthorized().body("User not authenticated");
        }
    };

    let mut content_items: Vec<ContentItem> = Vec::new();

    // Get text posts
    let text_posts_folder = format!("./user_pages/{}/text_posts", username);
    if let Ok(entries) = fs::read_dir(&text_posts_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(text_post) = serde_json::from_str::<TextPost>(&data) {
                        content_items.push(ContentItem::TextPost(text_post));
                    }
                }
            }
        }
    }

    // Get galleries
    let galleries_folder = format!("./user_pages/{}/gallery", username);
    if let Ok(entries) = fs::read_dir(&galleries_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let metadata_path = path.join("metadata.json");
                if metadata_path.exists() {
                    if let Ok(data) = fs::read_to_string(&metadata_path) {
                        if let Ok(gallery) = serde_json::from_str::<Gallery>(&data) {
                            content_items.push(ContentItem::Gallery(gallery));
                        }
                    }
                }
            }
        }
    }

    // Get films
    let films_folder = format!("./user_pages/{}/films", username);
    if let Ok(entries) = fs::read_dir(&films_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(film) = serde_json::from_str::<Film>(&data) {
                        content_items.push(ContentItem::Film(film));
                    }
                }
            }
        }
    }

    // Get audios
    let audios_folder = format!("./user_pages/{}/audios", username);
    if let Ok(entries) = fs::read_dir(&audios_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(audio) = serde_json::from_str::<Audio>(&data) {
                        content_items.push(ContentItem::Audio(audio));
                    }
                }
            }
        }
    }

    // Sort content items by timestamp in descending order
    content_items.sort_by(|a, b| {
        let timestamp_a = match a {
            ContentItem::TextPost(tp) => &tp.timestamp,
            ContentItem::Gallery(g) => &g.timestamp,
            ContentItem::Film(f) => &f.timestamp,
            ContentItem::Audio(aud) => &aud.timestamp,
        };
        let timestamp_b = match b {
            ContentItem::TextPost(tp) => &tp.timestamp,
            ContentItem::Gallery(g) => &g.timestamp,
            ContentItem::Film(f) => &f.timestamp,
            ContentItem::Audio(aud) => &aud.timestamp,
        };
        timestamp_b.cmp(timestamp_a)
    });

    HttpResponse::Ok().json(content_items)
}

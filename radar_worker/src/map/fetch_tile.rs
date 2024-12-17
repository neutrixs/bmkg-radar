use crate::map::{fake_headers::_fake_headers, MapImagery};
use bytes::Bytes;
use image::{DynamicImage, ImageReader};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::copy;
use std::path::PathBuf;
use url::Url;

#[derive(Hash)]
struct ContentHash {
    content: String,
}

fn _hash_tile_url(url: &String) -> String {
    let url = ContentHash {
        content: url.clone(),
    };
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:x}", hash)
}

fn _save_file(content: &Bytes, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut content = content.as_ref();
    let mut file = File::create(path)?;
    copy(&mut content, &mut file)?;
    Ok(())
}

fn _load_file(path: &PathBuf) -> Result<Bytes, Box<dyn Error>> {
    let file = fs::read(path)?;
    Ok(Bytes::from(file))
}

impl MapImagery {
    pub(crate) async fn fetch_tile(&self, x: i32, y: i32) -> Result<DynamicImage, Box<dyn Error>> {
        let url = self.style.url(x, y, self.zoom_level);
        let hash = _hash_tile_url(&url);
        let filename = format!("tile-{}", hash);

        let temp_dir = std::env::temp_dir();
        let full_path = temp_dir.join(&filename);

        if full_path.exists() {
            let file = _load_file(&full_path)?;
            let img = ImageReader::new(std::io::Cursor::new(file))
                .with_guessed_format()?
                .decode()?;
            return Ok(img);
        }
        let client = reqwest::Client::new();
        let response = client.get(&url).headers(_fake_headers()).send().await?;
        if !response.status().is_success() {
            return Err(format!(
                "Failed to fetch tile, {} {}: {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("unknown"),
                Url::parse(&url)
                    .expect("Failed to parse URL")
                    .host_str()
                    .unwrap_or("Unknown domain"),
            )
            .into());
        }

        let img = response.bytes().await?;
        let _ = _save_file(&img, &full_path);

        let img = ImageReader::new(std::io::Cursor::new(img))
            .with_guessed_format()?
            .decode()?;

        Ok(img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Coordinate;
    use rand::{rng, Rng};

    #[test]
    fn test_hash_tile_url() {
        let url = String::from("https://tile.openstreetmap.org/12/123/456.png");
        let hash = _hash_tile_url(&url);
        assert_eq!(hash, "198fba2219ee7da0");
    }

    #[test]
    fn test_save_load_file() {
        let random = rng().random::<f64>();
        let content = Bytes::from(random.to_string());
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("fetch_tile_save_test");
        let _ = _save_file(&content, &test_path);
        assert!(test_path.exists());

        let saved_content = fs::read(&test_path).expect("Failed to read saved file");
        assert_eq!(
            content, saved_content,
            "File content from _save_file does not match"
        );

        let load = _load_file(&test_path).expect("Failed to load file");
        assert_eq!(content, load, "File content from _load_file does not match");

        fs::remove_file(&test_path).expect("Failed to clean up test file");
    }

    #[tokio::test]
    async fn test_fetch_tile() {
        let imagery = MapImagery::builder([
            Coordinate { lat: 0., lon: 0. },
            Coordinate { lat: 1., lon: 1. },
        ])
        .build();
        let img = imagery.fetch_tile(0, 0).await;
        assert!(
            img.is_ok(),
            "fetch_tile returned an error: {:?}",
            img.unwrap_err()
        );
    }
}

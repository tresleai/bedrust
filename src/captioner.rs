use std::io::Write;
use std::{fs, io::Read, path::PathBuf};

use anyhow::anyhow;
use base64::{engine::general_purpose, Engine as _};

use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::se;
use serde::Serialize;

use crate::ask_bedrock;
use crate::RunType;

#[derive(Debug, Serialize)]
pub struct Image {
    pub path: PathBuf,
    #[serde(skip_serializing)]
    pub extension: String,
    // FIX: Think about setting the base64 as optional
    #[serde(skip_serializing)]
    pub base64: String,
    pub caption: Option<String>,
}

pub enum OutputFormat {
    Json,
    Xml,
}

impl Image {
    pub fn new(p: &PathBuf) -> Result<Self, anyhow::Error> {
        let extension = &p
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("The file provided has no extension, this is an issue."))?;
        let img_base64 = load_image(p)?;

        // FIX: Clone happens - see if we should remove that
        let image = Self {
            path: p.clone(),
            extension: extension.to_string(),
            base64: img_base64,
            caption: None,
        };
        Ok(image)
    }
}

pub fn list_files_in_path_by_extension(
    p: PathBuf,
    ext: Vec<String>,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let entries = fs::read_dir(p)?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("There was no files with extensions in this directory"))?;
        if ext.contains(&extension.to_string()) {
            files.push(path);
        }
    }
    Ok(files)
}

pub fn load_image(p: &PathBuf) -> Result<String, anyhow::Error> {
    let mut file = fs::File::open(p)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let base64_img = general_purpose::STANDARD.encode(buffer);
    Ok(base64_img)
}

pub async fn caption_image(
    i: &mut Vec<crate::captioner::Image>,
    model: &str,
    prompt: &String,
    runtime_client: &aws_sdk_bedrockruntime::Client,
    bedrock_client: &aws_sdk_bedrock::Client,
) -> Result<(), anyhow::Error> {
    // progress bar shenanigans
    let progress_bar = ProgressBar::new(i.len().try_into()?);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{wide_bar:.cyan/blue}] {msg} ({pos}/{len})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));

    for image in i {
        let message = image.path.as_path().display().to_string();
        progress_bar.set_message(message);
        let caption = ask_bedrock(
            prompt,
            Some(image),
            model,
            RunType::Captioning,
            runtime_client,
            bedrock_client,
        )
        .await?;
        progress_bar.inc(1);
        image.caption = Some(caption);
    }
    progress_bar.finish();

    Ok(())
}

pub fn write_captions(
    i: Vec<crate::captioner::Image>,
    format: OutputFormat,
    filename: &str,
) -> Result<(), anyhow::Error> {
    match format {
        OutputFormat::Json => {
            let mut json_file = std::fs::File::create(filename).expect("Failed to create file");
            let json_serialized =
                serde_json::to_string_pretty(&i).expect("Failed to serialize data");
            json_file
                .write_all(json_serialized.as_bytes())
                .expect("Failed to write to file");
        }
        OutputFormat::Xml => {
            let mut xml_file = std::fs::File::create(filename).expect("Failed to create file");
            let xmled = se::to_string_with_root("captions", &i).expect("Failed to convert to XML");
            xml_file
                .write_all(xmled.as_bytes())
                .expect("Failed to write to file");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_bedrust_config;
    use image::{Rgb, RgbImage};
    use rand::distributions::{Alphanumeric, DistString};

    // list all files in the directory
    #[test]
    fn list_all_images() {
        // prep the test by creating a know directory
        // generate random string for dir name
        let dir_prefix = Alphanumeric.sample_string(&mut rand::thread_rng(), 5);
        let dir_path = format!("/tmp/{}-bedrusttest", dir_prefix);
        // create the dir
        // TODO: handle issues creating the path
        fs::create_dir_all(&dir_path).unwrap();
        // creating files:
        let file1_path = format!("{}/file1.jpeg", &dir_path);
        let file2_path = format!("{}/123456789.md5", &dir_path);
        let file3_path = format!("{}/alanford.jpg", &dir_path);
        let file4_path = format!("{}/bobrok.png", &dir_path);
        let file5_path = format!("{}/superhik.bmp", &dir_path);
        // TODO: handle issues creating the path
        fs::File::create(&file1_path).unwrap();
        fs::File::create(file2_path).unwrap();
        fs::File::create(&file3_path).unwrap();
        fs::File::create(&file4_path).unwrap();
        fs::File::create(&file5_path).unwrap();

        // load supported file extensions
        let config = load_bedrust_config().unwrap();

        let list =
            list_files_in_path_by_extension(PathBuf::from(dir_path), config.supported_images);
        let expected_vec = vec![
            PathBuf::from(&file1_path),
            PathBuf::from(&file3_path),
            PathBuf::from(&file4_path),
            PathBuf::from(&file5_path),
        ];
        assert_eq!(expected_vec, list.unwrap());
    }

    #[test]
    fn load_image_from_disk() {
        let image_path = PathBuf::from("/tmp/bedrust_test_image.jpeg");
        // generate an image
        let mut img = RgbImage::new(32, 32);
        for x in 15..=17 {
            for y in 8..24 {
                img.put_pixel(x, y, Rgb([255, 0, 0]));
                img.put_pixel(y, x, Rgb([255, 0, 0]));
            }
        }
        img.save(&image_path).unwrap();

        // load the generated image from disk
        let test_image = load_image(&PathBuf::from(&image_path)).unwrap();

        // this is just raw base64 of the generated image from above
        let image_base64 = "/9j/4AAQSkZJRgABAgAAAQABAAD/wAARCAAgACADAREAAhEBAxEB/9sAQwAIBgYHBgUIBwcHCQkICgwUDQwLCwwZEhMPFB0aHx4dGhwcICQuJyAiLCMcHCg3KSwwMTQ0NB8nOT04MjwuMzQy/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwD5/oAKACgAoAKANKwsLxNRtWa0nVVmQkmMgAZHtXNWrU3TklJbPqe1l2XYyGMpSlSkkpR+y+68ixrlldzazcPHazOh24ZYyQflFZ4SrTjRScl9/mdvEGAxVXMak6dKTTtqk2vhXkYtdp8wFAG1Za5qM1/bxvcZR5VVhsXkE/SuKrhKMacml0fc+nwHEGY1cVSpzqXTkk9I7NryJ9Y1i/tdVmhhn2xrtwNin+EHuKzw2GpTpKUlr8+51Z3nePw2PqUaNS0VaysuyfVHPV6J8cFABQAUAFAH/9k=";
        assert_eq!(test_image, image_base64);
    }
}

use std::fs;
use std::path::{Path, PathBuf};
use clap::{Parser};
use image::{DynamicImage, ImageError, ImageFormat};
use image::io::{Reader as ImageReader};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    file: Option<String>,

    #[arg(long)]
    folder: Option<String>,

    #[arg(long)]
    from: Option<String>,

    #[arg(long)]
    to: String,
}

fn main() {
    let args = Args::parse();

    if let Some(file) = args.file{
        handle_image_file(&file, &args.to);
    }

    if let Some(folder) = args.folder{
        match args.from {
            Some(extension) => {
                handle_folder(&folder, &extension, &args.to);
            }
            None => {
                eprintln!("Expecting --from argument specifying extensions to convert from.");
            }
        }
    }
}

fn handle_folder(folder: &str, from: &str, to: &str) {
    let entries = fs::read_dir(folder).expect(&*format!("Failed to read directory: {folder}"));

    for entry in entries{
        if let Ok(entry) = entry{
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some(&*from) {
                let path_string = path.to_str().unwrap();
                handle_image_file(path_string, to);
            }
        }
    }

}

fn handle_image_file(file: &str, to: &str) {
    match load_image(&file){
        Ok(image) => {
            convert_image(&file, &image, &to);
        }
        Err(e) => {
            eprintln!("Failed to load image: {}", e);
        }
    }
}

fn convert_image(file: &str, image: &DynamicImage, to: &str) {
    let path = Path::new(file);
    let mut path_buf = PathBuf::from(path);
    path_buf.set_extension(to);

    let image_format = get_image_format(to);
    match image_format {
        Some(format) => {
            image.save_with_format(path_buf, format).expect(&*format!("Error when converting file: {file}"));
        }
        _ => {
            eprintln!("Unable to save image.");
        }
    }
}

fn get_image_format(format: &str) -> Option<ImageFormat>{
    match format {
        "png" => Some(ImageFormat::Png),
        "jpeg" | "jpg" => Some(ImageFormat::Jpeg),
        "webp" => Some(ImageFormat::WebP),
        "bmp" => Some(ImageFormat::Bmp),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "gif" => Some(ImageFormat::Gif),
        _ => {
            eprintln!("Unable to convert to '{}', this format is unsupported.", format);
            None
        }
    }
}

fn load_image(file: &str) -> Result<DynamicImage, ImageError> {
    ImageReader::open(file)?.decode()
}
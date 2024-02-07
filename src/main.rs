use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering::SeqCst;
use clap::{Parser};
use image::{DynamicImage, ImageError, ImageFormat};
use image::io::{Reader as ImageReader};
use rayon::prelude::*;

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

    #[arg(long, default_value_t = false)]
    all: bool
}

fn main() {
    let args = Args::parse();

    if let Some(file) = args.file{
        handle_image_file(&file, &args.to, 1, 1);
    }

    if let Some(folder) = args.folder{
        if args.all {
            handle_folder_all(&folder, &args.to);
            return;
        }

        match args.from {
            Some(extension) => {
                handle_folder(&folder, &extension, &args.to);
            }
            None => {
                eprintln!("ImageFlipper: [Expecting --from argument specifying extensions to convert from.]");
            }
        }
    }
}

fn handle_folder_all(folder: &str, to: &str){
    let entries = fs::read_dir(folder)
        .unwrap_or_else(|err| {
            eprintln!("ImageFlipper: [Failed to read directory: {}]", err);
            std::process::exit(1);
        })
        .map(|res| res.unwrap())
        .filter(|entry|
            entry.path().is_file() &&
            entry.path().extension().and_then(|ext| ext.to_str()) != Some(&*to) &&
            is_image_format(entry.path().extension().and_then(|ext| ext.to_str()))
        )
        .collect::<Vec<_>>();

    handle_image_files(&entries, to);
}

fn handle_folder(folder: &str, from: &str, to: &str) {
    let entries = fs::read_dir(folder)
        .unwrap_or_else(|err| {
            eprintln!("ImageFlipper: [Failed to read directory: {}]", err);
            std::process::exit(1);
        })
        .map(|res| res.unwrap())
        .filter(|entry|
            entry.path().is_file() &&
                entry.path().extension().and_then(|ext| ext.to_str()) == Some(&*from))
        .collect::<Vec<_>>();

    handle_image_files(&entries, to);
}

fn handle_image_files(entries: &Vec<DirEntry>, to: &str) {
    let counter = AtomicU16::new(0);
    let item_total_count = entries.len();
    entries.par_iter().for_each(|entry|{
        counter.fetch_add(1, SeqCst);
        let item_current_count = counter.load(SeqCst);
        let path = entry.path();
        let path_string = path.to_str().unwrap();
        handle_image_file(path_string, to, item_current_count, item_total_count as u16);
    });
}

fn is_image_format(extension: Option<&str>) -> bool {
    match extension {
        Some(ext) => {
            ext == "png" ||
            ext == "jpg" || ext == "jpeg" ||
            ext == "webp" ||
            ext == "bmp" ||
            ext == "tiff" || ext == "tif" ||
            ext == "gif" ||
            ext == "ico"
        }
        _ => {
            false
        }
    }
}

fn handle_image_file(file: &str, to: &str, item_current_count: u16, item_total_count: u16) {
    match load_image(&file){
        Ok(image) => {
            convert_image(&file, &image, &to, item_current_count, item_total_count);
        }
        Err(e) => {
            eprintln!("ImageFlipper: [Failed to load image: {}]", e);
        }
    }
}

fn convert_image(file: &str, image: &DynamicImage, to: &str, item_current_count: u16, item_total_count: u16) {
    let path = Path::new(file);
    let mut path_buf = PathBuf::from(path);
    path_buf.set_extension(to);

    let image_format = get_image_format(to);
    match image_format {
        Some(format) => {
            println!("{}", format!("ImageFlipper: [Converting image (#{} of {}) {:?} to .{}]", item_current_count, item_total_count, path_buf.file_stem().unwrap(), to));
            let rgb_image = image.to_rgb8();
            rgb_image.save_with_format(path_buf, format).expect(&*format!("ImageFlipper: [Error when converting file: {file}]"));
        }
        _ => {
            eprintln!("ImageFlipper: [Unable to save image.]");
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
        "ico" => Some(ImageFormat::Ico),
        _ => {
            eprintln!("ImageFlipper: [Unable to convert to '{}', this format is unsupported.]", format);
            None
        }
    }
}

fn load_image(file: &str) -> Result<DynamicImage, ImageError> {
    ImageReader::open(file)?.decode()
}
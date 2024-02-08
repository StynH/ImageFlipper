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
    ///Name of a single file to convert. (This or --folder is required.)
    #[arg(long)]
    file: Option<String>,

    ///Directory to convert images in. (This or --file is required.)
    #[arg(long)]
    folder: Option<String>,

    ///Output folder (Optional)
    #[arg(long, short)]
    output: Option<String>,

    ///Image format to convert all images in folder from. (Optional)
    #[arg(long)]
    from: Option<String>,

    ///Image format to convert to.
    #[arg(long, short)]
    to: String,

    ///Convert all images that are not the target format. (Optional)
    #[arg(long, short, default_value_t = false)]
    all: bool
}

fn main() {
    let args = Args::parse();
    let output = args.output;

    if let Some(file) = args.file{
        handle_image_file(&file, &args.to, 1, 1, &output);
    }

    if let Some(folder) = args.folder{
        if args.all {
            handle_folder_all(&folder, &args.to, &output);
            return;
        }

        match args.from {
            Some(extension) => {
                handle_folder(&folder, &extension, &args.to, &output);
            }
            None => {
                eprintln!("ImageFlipper: [Expecting --from argument specifying extensions to convert from.]");
            }
        }
    }
}

fn handle_folder_all(folder: &str, to: &str, output_folder: &Option<String>){
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

    handle_image_files(&entries, to, output_folder);
}

fn handle_folder(folder: &str, from: &str, to: &str, output_folder: &Option<String>) {
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

    handle_image_files(&entries, to, output_folder);
}

fn handle_image_files(entries: &Vec<DirEntry>, to: &str, output_folder: &Option<String>) {
    let counter = AtomicU16::new(0);
    let item_total_count = entries.len();
    entries.par_iter().for_each(|entry|{
        counter.fetch_add(1, SeqCst);
        let item_current_count = counter.load(SeqCst);
        let path = entry.path();
        let path_string = path.to_str().unwrap();
        handle_image_file(path_string, to, item_current_count, item_total_count as u16, output_folder);
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

fn handle_image_file(file: &str, to: &str, item_current_count: u16, item_total_count: u16, output_folder: &Option<String>) {
    match load_image(&file){
        Ok(image) => {
            convert_image(&file, &image, &to, item_current_count, item_total_count, output_folder);
        }
        Err(e) => {
            eprintln!("ImageFlipper: [Failed to load image: {}]", e);
        }
    }
}

fn convert_image(file: &str, image: &DynamicImage, to: &str, item_current_count: u16, item_total_count: u16, output_folder: &Option<String>) {
    let path = Path::new(file);
    let image_format = get_image_format(to);

    match image_format {
        Some(format) => {
            let rgb_image = image.to_rgb8();
            match output_folder {
                Some(output_path) => {
                    let mut new_path_buf = change_directory(path, Path::new(output_path));
                    new_path_buf.set_extension(to);
                    println!("{}", format!("ImageFlipper: [Converting image (#{} of {}) {:?} to .{}]", item_current_count, item_total_count, new_path_buf.file_stem().unwrap(), to));
                    rgb_image.save_with_format(new_path_buf, format).expect(&*format!("ImageFlipper: [Error when converting file: {file}]"));
                }
                None => {
                    let mut path_buf = PathBuf::from(path);
                    path_buf.set_extension(to);
                    println!("{}", format!("ImageFlipper: [Converting image (#{} of {}) {:?} to .{}]", item_current_count, item_total_count, path_buf.file_stem().unwrap(), to));
                    rgb_image.save_with_format(path_buf, format).expect(&*format!("ImageFlipper: [Error when converting file: {file}]"));
                }
            }
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

fn change_directory(path: &Path, new_dir: &Path) -> PathBuf {
    let file_name = path.file_name().expect("ImageFlipper: [Original path does not have a file name]");

    if new_dir.is_absolute() {
        let mut new_path = PathBuf::from(new_dir);
        new_path.push(file_name);
        create_directory_if_not_exists(&new_path);
        new_path
    } else {
        let mut new_path = path
            .parent()
            .expect("ImageFlipper: [Original path does not have a parent directory]")
            .to_path_buf();

        new_path.push(new_dir);
        create_directory_if_not_exists(&new_path);
        new_path.push(file_name);
        new_path
    }
}

fn create_directory_if_not_exists(path: &PathBuf) {
    if !Path::new(&path).exists() {
        fs::create_dir_all(path).expect("ImageFlipper: [Unable to create missing output directory]");
    }
}
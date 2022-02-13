use anyhow::Result;
use clap::{ArgEnum, Parser};
use image::io::Reader as ImageReader;
use image::{GrayImage, Luma};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[clap(parse(from_str))]
    input_path: PathBuf,
    #[clap(parse(from_str))]
    output_path: PathBuf,
    #[clap(short,long, arg_enum, default_value_t = Mode::Naive1d)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, ArgEnum)]
enum Mode {
    Naive1d,
    Naive2d,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut img = ImageReader::open(args.input_path)?.decode()?.to_luma8();

    match args.mode {
        Mode::Naive1d => naive_1d_error_diffusion(&mut img),
        Mode::Naive2d => naive_2d_error_diffusion(&mut img),
    }

    img.save(args.output_path)?;
    Ok(())
}

fn naive_1d_error_diffusion(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut quantization_error;

        for x in 0..width - 1 {
            quantization_error = quantize_pixel(img, x, y);
            diffuse_error_to_pixel(img, x + 1, y, quantization_error);
        }
    }
}

fn naive_2d_error_diffusion(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut quantization_error;

        for x in 0..width - 1 {
            quantization_error = quantize_pixel(img, x, y);
            diffuse_error_to_pixel(img, x + 1, y, quantization_error / 2);
            diffuse_error_to_pixel(img, x, y + 1, quantization_error / 2);
        }
    }
}

fn quantize_pixel(img: &mut GrayImage, x: u32, y: u32) -> i16 {
    let old_intensity = img.get_pixel(x, y)[0];
    let new_intensity;
    let quantization_error;

    if old_intensity > 127 {
        new_intensity = 255;
        quantization_error = i16::from(old_intensity) - 255;
    } else {
        new_intensity = 0;
        quantization_error = i16::from(old_intensity);
    }

    let new_pixel = Luma::<u8>([new_intensity]);
    img.put_pixel(x, y, new_pixel);

    quantization_error
}

fn diffuse_error_to_pixel(img: &mut GrayImage, x: u32, y: u32, err: i16) {
    let old_intensity = img.get_pixel(x, y)[0];
    let new_intensity = coerce_to_u8(i16::from(old_intensity) + err);
    let new_pixel = Luma::<u8>([new_intensity]);
    img.put_pixel(x, y, new_pixel);
}

fn coerce_to_u8(i: i16) -> u8 {
    if i > i16::from(std::u8::MAX) {
        std::u8::MAX
    } else if i < i16::from(std::u8::MIN) {
        std::u8::MIN
    } else {
        i as u8
    }
}

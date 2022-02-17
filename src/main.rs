use anyhow::Result;
use clap::{ArgEnum, Parser};
use image::io::Reader as ImageReader;
use image::{GenericImageView, GrayImage, Luma};
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
    NaiveQuantization,
    Naive1d,
    Naive2d,
    FloydSteinberg,
    Atkinson,
    Sierra,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut img = ImageReader::open(args.input_path)?.decode()?.to_luma8();

    match args.mode {
        Mode::NaiveQuantization => naive_quantization(&mut img),
        Mode::Naive1d => naive_1d_dithering(&mut img),
        Mode::Naive2d => naive_2d_dithering(&mut img),
        Mode::FloydSteinberg => floyd_steinberg_dithering(&mut img),
        Mode::Atkinson => atkinson_dithering(&mut img),
        Mode::Sierra => sierra_dithering(&mut img),
    }

    img.save(args.output_path)?;
    Ok(())
}

fn naive_quantization(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            quantize_pixel(img, x, y);
        }
    }
}

fn naive_1d_dithering(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut quantization_error;

        for x in 0..width - 1 {
            quantization_error = quantize_pixel(img, x, y);
            diffuse_error_to_pixel(img, x + 1, y, quantization_error, 1, 1);
        }
    }
}

fn naive_2d_dithering(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut quantization_error;

        for x in 0..width - 1 {
            quantization_error = quantize_pixel(img, x, y);
            diffuse_error_to_pixel(img, x, y + 1, quantization_error, 1, 2);
            diffuse_error_to_pixel(img, x + 1, y, quantization_error, 1, 2);
        }
    }
}

fn floyd_steinberg_dithering(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut quantization_error;

        for x in 0..width - 1 {
            quantization_error = quantize_pixel(img, x, y);
            diffuse_error_to_pixel(img, x, y + 1, quantization_error, 5, 16);
            diffuse_error_to_pixel(img, x + 1, y, quantization_error, 7, 16);
            diffuse_error_to_pixel(img, x + 1, y + 1, quantization_error, 1, 16);
            if x > 0 {
                diffuse_error_to_pixel(img, x - 1, y + 1, quantization_error, 3, 16);
            }
        }
    }
}

fn atkinson_dithering(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut quantization_error;

        for x in 0..width - 1 {
            quantization_error = quantize_pixel(img, x, y);
            diffuse_error_to_pixel(img, x, y + 1, quantization_error, 1, 8);
            diffuse_error_to_pixel(img, x, y + 2, quantization_error, 1, 8);
            diffuse_error_to_pixel(img, x + 1, y, quantization_error, 1, 8);
            diffuse_error_to_pixel(img, x + 1, y + 1, quantization_error, 1, 8);
            diffuse_error_to_pixel(img, x + 2, y, quantization_error, 1, 8);
            if x > 0 {
                diffuse_error_to_pixel(img, x - 1, y + 1, quantization_error, 1, 8);
            }
        }
    }
}

fn sierra_dithering(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut quantization_error;

        for x in 0..width - 1 {
            quantization_error = quantize_pixel(img, x, y);
            diffuse_error_to_pixel(img, x, y + 1, quantization_error, 5, 32);
            diffuse_error_to_pixel(img, x, y + 2, quantization_error, 3, 32);
            diffuse_error_to_pixel(img, x + 1, y, quantization_error, 5, 32);
            diffuse_error_to_pixel(img, x + 1, y + 1, quantization_error, 4, 32);
            diffuse_error_to_pixel(img, x + 1, y + 2, quantization_error, 2, 32);
            diffuse_error_to_pixel(img, x + 2, y, quantization_error, 3, 32);
            diffuse_error_to_pixel(img, x + 2, y + 1, quantization_error, 2, 32);
            if x > 0 {
                diffuse_error_to_pixel(img, x - 1, y + 1, quantization_error, 4, 32);
                diffuse_error_to_pixel(img, x - 1, y + 1, quantization_error, 2, 32);
            }

            if x > 1 {
                diffuse_error_to_pixel(img, x - 2, y + 1, quantization_error, 2, 32);
            }
        }
    }
}

fn quantize_pixel(img: &mut GrayImage, x: u32, y: u32) -> i16 {
    let old_intensity = img.get_pixel(x, y)[0];
    let new_intensity;

    if old_intensity > 127 {
        new_intensity = 255;
    } else {
        new_intensity = 0;
    }

    let new_pixel = Luma::<u8>([new_intensity]);
    img.put_pixel(x, y, new_pixel);

    i16::from(old_intensity) - i16::from(new_intensity)
}

fn diffuse_error_to_pixel(
    img: &mut GrayImage,
    x: u32,
    y: u32,
    err: i16,
    factor_numerator: i16,
    factor_denominator: i16,
) {
    if img.in_bounds(x, y) {
        let old_intensity = img.get_pixel(x, y)[0];
        let new_intensity =
            coerce_to_u8(i16::from(old_intensity) + err * factor_numerator / factor_denominator);
        let new_pixel = Luma::<u8>([new_intensity]);
        img.put_pixel(x, y, new_pixel);
    }
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

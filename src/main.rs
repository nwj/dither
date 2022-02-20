use anyhow::Result;
use clap::{ArgEnum, Parser};
use image::io::Reader as ImageReader;
use image::{GenericImageView, GrayImage, Luma};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Input image file. Most common image formats (jpg, png, etc.) are supported.
    #[clap(parse(from_str))]
    input_path: PathBuf,
    /// Output image file. Any pre-existing file here will be overwritten.
    #[clap(parse(from_str))]
    output_path: PathBuf,
    /// Specify the dithering algorithm that will be applied.
    #[clap(short,long, arg_enum, default_value_t = Mode::FloydSteinberg)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, ArgEnum)]
enum Mode {
    Quantization,
    Random,
    Naive1d,
    Naive2d,
    FloydSteinberg,
    FalseFloydSteinberg,
    JarvisJudiceNinke,
    Stucki,
    Atkinson,
    Burkes,
    Sierra,
    TwoRowSierra,
    SierraLite,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut img = ImageReader::open(args.input_path)?.decode()?.to_luma8();

    match args.mode {
        Mode::Quantization => quantization(&mut img),
        Mode::Random => random_quantization(&mut img),
        Mode::Naive1d => naive_1d_dithering(&mut img),
        Mode::Naive2d => naive_2d_dithering(&mut img),
        Mode::FloydSteinberg => floyd_steinberg_dithering(&mut img),
        Mode::FalseFloydSteinberg => false_floyd_steinberg_dithering(&mut img),
        Mode::JarvisJudiceNinke => jarvis_judice_ninke_dithering(&mut img),
        Mode::Stucki => stucki_dithering(&mut img),
        Mode::Atkinson => atkinson_dithering(&mut img),
        Mode::Burkes => burkes_dithering(&mut img),
        Mode::Sierra => sierra_dithering(&mut img),
        Mode::TwoRowSierra => two_row_sierra_dithering(&mut img),
        Mode::SierraLite => sierra_lite_dithering(&mut img),
    }

    img.save(args.output_path)?;
    Ok(())
}

type Delta = (i32, i32);
type Ratio = (i16, i16);
struct Coord(Delta, Ratio);

fn quantization(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            quantize_pixel(img, x, y);
        }
    }
}

fn random_quantization(img: &mut GrayImage) {
    let mut rng = SmallRng::from_entropy();
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            quantize_pixel_with_rng(&mut rng, img, x, y);
        }
    }
}

fn naive_1d_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [Coord((1, 0), (1, 1))];
    generic_dithering(img, &diffusion_matrix);
}

fn naive_2d_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [Coord((1, 0), (1, 2)), Coord((0, 1), (1, 2))];
    generic_dithering(img, &diffusion_matrix);
}

fn floyd_steinberg_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (7, 16)),
        Coord((0, 1), (5, 16)),
        Coord((1, 1), (1, 16)),
        Coord((-1, 1), (3, 16)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn false_floyd_steinberg_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (3, 8)),
        Coord((0, 1), (3, 8)),
        Coord((1, 1), (2, 8)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn jarvis_judice_ninke_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (7, 48)),
        Coord((2, 0), (5, 48)),
        Coord((0, 1), (7, 48)),
        Coord((0, 2), (5, 48)),
        Coord((1, 1), (5, 48)),
        Coord((1, 2), (3, 48)),
        Coord((2, 1), (3, 48)),
        Coord((2, 2), (1, 48)),
        Coord((-1, 1), (5, 48)),
        Coord((-1, 2), (3, 48)),
        Coord((-2, 1), (3, 48)),
        Coord((-2, 2), (1, 48)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn stucki_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (8, 42)),
        Coord((2, 0), (4, 42)),
        Coord((0, 1), (8, 42)),
        Coord((0, 2), (4, 42)),
        Coord((1, 1), (4, 42)),
        Coord((1, 2), (2, 42)),
        Coord((2, 1), (2, 42)),
        Coord((2, 2), (1, 42)),
        Coord((-1, 1), (4, 42)),
        Coord((-1, 2), (2, 42)),
        Coord((-2, 1), (2, 42)),
        Coord((-2, 2), (1, 42)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn atkinson_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (1, 8)),
        Coord((2, 0), (1, 8)),
        Coord((0, 1), (1, 8)),
        Coord((0, 2), (1, 8)),
        Coord((1, 1), (1, 8)),
        Coord((-1, 1), (1, 8)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn burkes_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (8, 32)),
        Coord((2, 0), (4, 32)),
        Coord((0, 1), (8, 32)),
        Coord((1, 1), (4, 32)),
        Coord((2, 1), (2, 32)),
        Coord((-1, 1), (4, 32)),
        Coord((-2, 1), (2, 32)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn sierra_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (5, 32)),
        Coord((2, 0), (3, 32)),
        Coord((0, 1), (5, 32)),
        Coord((0, 2), (3, 32)),
        Coord((1, 1), (4, 32)),
        Coord((2, 1), (2, 32)),
        Coord((1, 2), (2, 32)),
        Coord((-1, 1), (4, 32)),
        Coord((-1, 2), (2, 32)),
        Coord((-2, 1), (2, 32)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn two_row_sierra_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (4, 16)),
        Coord((2, 0), (3, 16)),
        Coord((0, 1), (3, 16)),
        Coord((1, 1), (2, 16)),
        Coord((2, 1), (1, 16)),
        Coord((-1, 1), (2, 16)),
        Coord((-2, 1), (1, 16)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn sierra_lite_dithering(img: &mut GrayImage) {
    let diffusion_matrix = [
        Coord((1, 0), (2, 4)),
        Coord((0, 1), (1, 4)),
        Coord((-1, 1), (1, 4)),
    ];
    generic_dithering(img, &diffusion_matrix);
}

fn quantize_pixel(img: &mut GrayImage, x: u32, y: u32) -> i16 {
    let old_intensity = img.get_pixel(x, y)[0];
    let new_intensity;

    if old_intensity < 128 {
        new_intensity = 0;
    } else {
        new_intensity = 255;
    }

    let new_pixel = Luma::<u8>([new_intensity]);
    img.put_pixel(x, y, new_pixel);

    i16::from(old_intensity) - i16::from(new_intensity)
}

fn quantize_pixel_with_rng(mut rng: impl rand::Rng, img: &mut GrayImage, x: u32, y: u32) -> i16 {
    let old_intensity = img.get_pixel(x, y)[0];
    let random_intensity = rng.gen_range(0..255);
    let new_intensity;

    if random_intensity > old_intensity {
        new_intensity = 0;
    } else {
        new_intensity = 255;
    }

    let new_pixel = Luma::<u8>([new_intensity]);
    img.put_pixel(x, y, new_pixel);

    i16::from(old_intensity) - i16::from(new_intensity)
}

fn generic_dithering(img: &mut GrayImage, diffusion_matrix: &[Coord]) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let quant_err = quantize_pixel(img, x, y);

            for &Coord((delta_x, delta_y), (numerator, denominator)) in diffusion_matrix {
                if let (Some(new_x), Some(new_y)) = (
                    checked_add_signed(x, delta_x),
                    checked_add_signed(y, delta_y),
                ) {
                    diffuse_error_to_pixel(img, new_x, new_y, quant_err, numerator, denominator)
                }
            }
        }
    }
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

fn checked_add_signed(a: u32, b: i32) -> Option<u32> {
    if b.is_positive() {
        a.checked_add(b as u32)
    } else {
        a.checked_sub(b.abs() as u32)
    }
}

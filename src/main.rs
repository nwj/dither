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

type Delta = (i32, i32);
type Ratio = (i16, i16);
struct Coord(Delta, Ratio);

fn naive_quantization(img: &mut GrayImage) {
    generic_dithering(img, &[]);
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

fn checked_add_signed(a: u32, b: i32) -> Option<u32> {
    if b.is_positive() {
        a.checked_add(b as u32)
    } else {
        a.checked_sub(b.abs() as u32)
    }
}

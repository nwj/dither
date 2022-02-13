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
        Mode::Naive1d => naive_1d_dither(&mut img),
        Mode::Naive2d => {}
    }

    img.save(args.output_path)?;
    Ok(())
}

fn naive_1d_dither(img: &mut GrayImage) {
    let (width, height) = img.dimensions();

    for y in 0..height - 1 {
        let mut err = 0;

        for x in 0..width - 1 {
            let old_intensity = img.get_pixel(x, y)[0];
            let new_intensity;
            if coerce_to_u8(i16::from(old_intensity) + err) > 127 {
                new_intensity = 255;
                err = i16::from(old_intensity) + err - 255;
            } else {
                new_intensity = 0;
                err = i16::from(old_intensity) + err;
            }

            let new_pixel = Luma::<u8>([new_intensity]);
            img.put_pixel(x, y, new_pixel);
        }
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

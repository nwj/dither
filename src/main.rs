use anyhow::Result;
use clap::Parser;
use image::io::Reader as ImageReader;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[clap(parse(from_str))]
    input_path: PathBuf,
    #[clap(parse(from_str))]
    output_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let img = ImageReader::open(args.input_path)?.decode()?;
    img.save(args.output_path)?;

    Ok(())
}

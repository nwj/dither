use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[clap(parse(from_str))]
    input_path: PathBuf,
    #[clap(parse(from_str))]
    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("in: {:?}, out: {:?}", args.input_path, args.output_path);
}

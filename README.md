# dither

An implementation of various image dithering algorithms (and related ideas like random quantization) in Rust, behind a CLI.

## Installation

`dither` is written in Rust, so you'll need to [install that](https://www.rust-lang.org/tools/install) first in order to compile it.

To build:

```
$ git clone git@github.com:nwj/dither.git
$ cd dither
$ cargo build --release
$ ./target/release/dither
```

For convenience, you can then copy or otherwise add `./target/release/dither` to your `$PATH`.

## Usage

The CLI can be called with a single path argument pointing to an image you'd like to dither:

```
dither image.jpg
```

This will default to using the Floyd-Steinberg dithering algorithm and will save the results to a new file that has been named accordingly
```
> dither image.jpg

> ls
image-floyd-steinberg.png       image.jpg

```

Optionally, the CLI accepts a `--mode` (or `-m`) flag which allows you to specify a different dithering algorithm and an optional second positional argument to specify a desired output file name.

```
> dither -m atkinson image.jpg output.jpg

> ls
image.jpg       output.jpg
```

A full list of supported dithering algorithms and their corresponding `--mode` arguments is available via `dither --help`.

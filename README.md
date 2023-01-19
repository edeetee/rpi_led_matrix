# Program to control leds through an ArtNet led strip controller.
This project is made for the purposes of making turning a bunch of WS2812 matrixes running off a raspberry pi into a reactive robot mouth.

## Usage
- [Install the rust toolchain](https://www.rust-lang.org/tools/install)
- Update the networks constants at the top of main.rs
- Update the mappings in main.rs
- Execute '```Cargo run```'

## Technology
Rust is the programming language

I use a variety of libraries to support the serialization, color and vector math, visible in Cargo.toml.
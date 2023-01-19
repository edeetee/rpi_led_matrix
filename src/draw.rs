use std::time::Duration;

use glam::Vec2;
use palette::{rgb::Rgb, FromColor, Hsv};

use crate::{mapping::LedMapping, matrix_mapping::LedSquare};

pub fn tri(pos: Vec2) -> f32 {
    // let length = pos.length();

    let abs = pos.abs();
    let added = abs.x+abs.y;

    added
}

pub struct Context {
    pub elapsed: Duration,
    pub elapsed_seconds: f32
}

pub fn draw(pos_f: Vec2, ctx: &Context) -> Rgb<palette::encoding::Srgb, u8>  {
    let scale = Vec2::new(0.2, 0.05);
    let animated_scale = Vec2::new(1.0+(((ctx.elapsed_seconds*3.44512).sin())*5.123).sin(), 1.0);

    let tri_pos = pos_f*scale*animated_scale;

    let shape_val = tri(tri_pos);

    let mask = if (0.0..3.0).contains(&shape_val) {
        1.0
    } else {
        0.0
    };

    // let hue
    let brightness = 1.0*mask;

    // (0.0..0.5)

    let hsv = Hsv::new(
        (shape_val*0.2)*360.0, 
        (shape_val%1.0).powf(0.01), 
        (shape_val%1.0).powf(1.0) * brightness
    );

    Rgb::from_color(hsv).into_format::<u8>()
}
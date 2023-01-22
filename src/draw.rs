use std::time::Duration;

use glam::Vec2;
use palette::{rgb::Rgb, FromColor, Hsv};

pub fn tri(pos: Vec2) -> f32 {
    // let length = pos.length();

    let abs = pos.abs();
    let added = abs.x + abs.y;

    added
}

pub struct Context {
    pub elapsed: Duration,
    pub elapsed_seconds: f32,
}

pub fn draw(pos: Vec2, ctx: &Context) -> Rgb<palette::encoding::Srgb, u8> {
    let scale = Vec2::new(0.2, 0.05);
    
    let animation_pos = ctx.elapsed_seconds - (pos*Vec2::new(3.0*(1.0+(0.423*ctx.elapsed_seconds).sin()),1.0)).length() * 0.055;
    let animated_scale = Vec2::new(1.1 + (((animation_pos.sin() * 1.44512).sin()) * 5.123).sin(), 1.0);

    let tri_pos = pos * scale * animated_scale;

    let mask_max = 3.0;
    let shape_val = tri(tri_pos) / mask_max;

    let mask = if (0.0..1.0).contains(&shape_val) {
        1.0
    } else {
        0.0
    };

    // let hue
    let brightness = 1.0 * mask;

    let hsv = Hsv::new(
        (((shape_val * 5.0 + ctx.elapsed_seconds * 20.0).sin()) * 0.2 + 0.8) * 360.0,
        1.0, // ((shape_val)%1.0).powf(0.5),
        ((shape_val*2.0 + ctx.elapsed_seconds * 0.51232) % 1.0).powi(2) * brightness,
    );

    Rgb::from_color(hsv).into_format::<u8>()
}

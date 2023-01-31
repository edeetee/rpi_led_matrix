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
    pub audio: Vec<f32>
}

pub fn draw(pos: Vec2, ctx: &Context) -> Rgb<palette::encoding::Srgb, u8> {
    let audio_val = if !ctx.audio.is_empty() {
        let audio_scale = Vec2::new(1.0/(16.0*4.0), 1.0/(16.0/2.0));

        let audio_sample_pos = pos*audio_scale;
        let audio_sample_f = audio_sample_pos.length();
        let audio_sample_index = (ctx.audio.len() as f32*audio_sample_f) as usize;
        ctx.audio[audio_sample_index.min(ctx.audio.len()-1)]
    } else {
        0.0
    };

    let scale = Vec2::new(0.2, 0.05);
    
    let animation_pos = ctx.elapsed_seconds*0.345 - (pos*Vec2::new(1.0*(1.0+(0.123*ctx.elapsed_seconds).sin()),1.0)).length() * 0.055;
    
    let fact_sin = (((animation_pos.sin() * 1.44512).sin()) * 5.123).sin();
    let animated_scale = Vec2::new(1.1 + 0.3 * fact_sin, 1.0);

    let tri_pos = pos * scale * animated_scale+pos.signum()*audio_val*1.0;

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
        (((shape_val * 10.0 + ctx.elapsed_seconds * 20.0).sin()) * 0.2 + 0.8) * 360.0,
        1.0, // ((shape_val)%1.0).powf(0.5),
        ((shape_val*1.0 + ctx.elapsed_seconds * 0.0) % 1.0).powi(4) * brightness,
    );

    Rgb::from_color(hsv).into_format::<u8>()
}

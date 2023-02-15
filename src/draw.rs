use std::{time::Duration, f32::consts::{*}, num};

use ecolor::{Hsva, Color32, Rgba, HsvaGamma};
use glam::Vec2;
use noise::{NoiseFn, Perlin};
use palette::{rgb::Rgb, FromColor, Hsv, Srgb, LinSrgb, IntoColor};

pub fn tri(pos: Vec2) -> f32 {
    // let length = pos.length();

    let abs = pos.abs();
    let added = abs.x + abs.y;

    added
}

trait Limitable {
    fn zigzag(&self) -> Self;
}

impl Limitable for f32 {
    fn zigzag(&self) -> Self {
        // let half_other = other/2.0;
        // let double_other = other*2.0;
        // let modded = self % double_other;
        // let fract_p = self/period;

        // (-half_other + modded) / half_other
        2.0*(self-(self+0.5).floor()).abs()
    }
}

pub struct DrawContext<'a> {
    pub elapsed: Duration,
    pub elapsed_seconds: f32,
    pub audio: &'a [f32],
}

impl DrawContext<'_> {
    fn sample_audio(&self, pos: Vec2) -> f32 {
        if !self.audio.is_empty() {
            let audio_scale = Vec2::new(1.0, 1.0) /(16.0*4.0);
    
            let audio_sample_pos = pos*audio_scale;
            let audio_sample_f = audio_sample_pos.length();
            let audio_sample_index = (self.audio.len() as f32*audio_sample_f) as usize;
    
            self.audio[audio_sample_index.min(self.audio.len()-1)]
        } else {
            0.0
        }
    }
}

pub fn draw_blobs(ctx: &DrawContext, pos: Vec2) -> Rgba {
    let audio_val = ctx.sample_audio(pos).clamp(-1.0, 1.0);

    let scale = Vec2::new(0.15, 0.05);
    
    //use this to have the animation animate over the distance
    let offset = 0.123*ctx.elapsed_seconds;
    let offset_val = offset.sin();

    let animation_pos = pos * Vec2::new(1.0*(1.0+offset_val),1.0);

    // let animation_f = ctx.elapsed_seconds*1.545 + noise::Perlin::default().get(ctx.elapsed_seconds);
    let animation_f = animation_pos.length()/16.0;

    // let noise = ctx.noise.get([animation_f as f64 * 5.0, 0.0]) as f32 * 2.0;
    let fact_sin = ((animation_f*3.32).sin() * 1.94512).sin();
    let animated_scale = Vec2::new(1.1 + 0.5 * fact_sin, 1.0);

    let tri_pos = pos * scale * Vec2::new(1.0+audio_val*3.0, 1.0);
        // + pos.signum() * Vec2::new(1.0,0.0)*audio_val*1.0;

    let mask_max = 3.0;
    let shape_val = tri(tri_pos) / mask_max;

    let mask = if (0.0..1.0).contains(&shape_val) {
        1.0
    } else {
        0.0
    };

    // let hue
    // let brightness = 1.0;

    let bit_destruction = (
        (pos.length()*100.0+ctx.elapsed_seconds*0.4) % 1.0) 
        // * 0.3 
        * (1.0 + (pos.length()*0.5-ctx.elapsed_seconds*4.0).sin().powf(2.0)
    );
    
    let val = (
        (
            shape_val*1.5
            + tri_pos.length()*0.1
            + ctx.elapsed_seconds * 0.1123
        ) % 1.0
        - bit_destruction * 0.5
    )
    .powi(3);

    // let val = audio_val;

    let hsv = HsvaGamma {
        h: ((shape_val * 20.0 + -ctx.elapsed_seconds * 3.1232).sin()) * 0.2 + 0.8,
        s: 1.0,
        v: val,
        a: 1.0
    };

    // let hsv = HsvaGamma::new(
    //     ,
    //     1.0, // ((shape_val)%1.0).powf(0.5),
    //     val * brightness,
    //     1.0
    // );

    hsv.into()
}

pub fn draw_lightning(ctx: &DrawContext, pos: Vec2) -> Rgba {
    let audio_val = ctx.sample_audio(pos*0.5);
    // let audio_val = 1.0;

    let scaled_pos = pos / (Vec2::ONE*32.0);
    let pos_length = scaled_pos.length();

    let wide_angle = pos.angle_between(Vec2::Y)/TAU*((3.0+(ctx.elapsed_seconds*4.2312).sin())*1.5);

    // let wobble = scaled_pos.length()*(ctx.elapsed_seconds*TAU).sin();

    let chaos_angle = pos.angle_between(Vec2::Y)/TAU*10.0;

    let main_line = ((
        wide_angle
        + ctx.elapsed_seconds*1.32
        // + audio_val*0.1
        + ( -ctx.elapsed_seconds*3.232 + (pos_length*5.412).sin() + pos_length*2.0 ).zigzag()*0.5
    ) % 1.0).abs();

    let fine_detail_pre_abs = chaos_angle
        + audio_val*0.3
        + ( pos_length*15.0 + ctx.elapsed_seconds*5.2 ).sin();

    let fine_detail = (fine_detail_pre_abs % 1.0).abs();

    let radial_line: f32 = if (
        main_line < 0.2
        && fine_detail < 0.5
        && 0.05 < audio_val.abs()
    ) {
        1.0
    } else {
        0.0
    };
    
    let fade_out = (1.0-scaled_pos.length()).max(0.0).powf(0.5);

    let hsv = HsvaGamma {
        h: 0.6 + fine_detail_pre_abs % 0.2,
        s: (pos_length).max(0.0).powf(0.5),
        v: 1.0,
        a: radial_line * fade_out
    };

    hsv.into()
    
    // Rgba::WHITE * radial_line * fade_out
}
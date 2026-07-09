use std::f32::consts::TAU;

use macroquad::prelude::*;
use nalgebra::DVector;

pub fn color_from_hue(hue: f32) -> Color {
    let kr = f32::fract((5.0 + hue * 6.0) / 6.0) * 6.0;
    let kg = f32::fract((3.0 + hue * 6.0) / 6.0) * 6.0;
    let kb = f32::fract((1.0 + hue * 6.0) / 6.0) * 6.0;
    
    let r = 1.0 - f32::max(f32::min(f32::min(kr, 4.0 - kr), 1.0), 0.0);
    let g = 1.0 - f32::max(f32::min(f32::min(kg, 4.0 - kg), 1.0), 0.0);
    let b = 1.0 - f32::max(f32::min(f32::min(kb, 4.0 - kb), 1.0), 0.0);
    
    return Color::new(r, g, b, 1.0);
}

pub fn color_from_wv(vector: &DVector<f32>, w_scale: f32, edge_color: Color) -> Color {
    if vector.len() < 4 {
        return edge_color;
    }
    
    let wv_vector = Vec2::new(vector[3], {
        if vector.len() == 4 {
            0.0
        } else {
            vector[4]
        }
    });
    
    let fade_to_color = color_from_hue((wv_vector.to_angle() / TAU) + 0.5 + (1.0 / 12.0));
    let fade_strength = f32::min(wv_vector.length() * w_scale, 1.0);
    
    return Color::new(
        f32::lerp(edge_color.r, fade_to_color.r, (fade_strength * 2.0).min(1.0)),
        f32::lerp(edge_color.g, fade_to_color.g, (fade_strength * 2.0).min(1.0)),
        f32::lerp(edge_color.b, fade_to_color.b, (fade_strength * 2.0).min(1.0)),
        1.0 - fade_strength
    );
}

pub fn fade_from_depth(z: f32, near: f32, far: f32, zoom: f32) -> f32 {
    1.0 - clamp(f32::inverse_lerp(near + zoom, far + zoom, z), 0.0, 1.0)
}
use macroquad::prelude::*;
use nalgebra::{DMatrix, DVector};

use crate::color::*;
use crate::math::*;
use crate::scene::*;

pub fn draw_triangle_color(v1: Vec2, v2: Vec2, v3: Vec2, color1: Color, color2: Color, color3: Color) {
    let context = unsafe { get_internal_gl() };

    let vertices = [
        Vertex::new(v1.x, v1.y, 0., 0., 0., color1),
        Vertex::new(v2.x, v2.y, 0., 0., 0., color2),
        Vertex::new(v3.x, v3.y, 0., 0., 0., color3),
    ];

    let indices: [u16; 3] = [0, 1, 2];

    context.quad_gl.texture(None);
    context.quad_gl.draw_mode(DrawMode::Triangles);
    context.quad_gl.geometry(&vertices, &indices);
}

pub fn draw_variable_width_line(start_point: Vec2, end_point: Vec2, start_radius: f32, end_radius: f32, start_color: Color, end_color: Color) {
    if start_color.a > 0.0 || end_color.a > 0.0 {
        let edge_direction = (end_point - start_point).normalize();
        let left_of_edge = vec2(edge_direction.y, -edge_direction.x);
        let right_of_edge = vec2(-edge_direction.y, edge_direction.x);
        
        draw_triangle_color(
            start_point + (left_of_edge * start_radius),
            start_point + (right_of_edge * start_radius),
            end_point + (left_of_edge * end_radius),
            start_color,
            start_color,
            end_color,
        );
        
        draw_triangle_color(
            end_point + (left_of_edge * end_radius),
            end_point + (right_of_edge * end_radius),
            start_point + (right_of_edge * start_radius),
            end_color,
            end_color,
            start_color,
        );
    }
}

pub fn render(scene: &Scene, subdivisions: i32, shape_matrix: &DMatrix<f32>, shape_position: &DVector<f32>, edge_width: f32, near: f32, far: f32, zoom: f32, w_scale: f32, render_size: f32, screen_size: &Vec2) {
    clear_background(BLACK);
    
    let mut local_space_vertices: Vec<DVector<f32>> = Vec::new();

    for vertex in &scene.vertices {
        // Vertex in world/camera space
        let transformed_vertex = (shape_matrix * vertex) + shape_position;
        
        // Store vertex result
        local_space_vertices.push(transformed_vertex);
    }
    
    for i in (0..scene.edges.len()).step_by(2) {
        // A and B are the ends of the edges, 1 and 2 are the ends of the sub edges
        let vertex_a = &local_space_vertices[scene.edges[i]];
        let vertex_b = &local_space_vertices[scene.edges[i + 1]];
        
        for s in 0..subdivisions {
            let vertex_1 = vertex_a.lerp(&vertex_b, (s as f32) / (subdivisions as f32));
            let vertex_2 = vertex_a.lerp(&vertex_b, ((s + 1) as f32) / (subdivisions as f32));
            
            let radius_1 = (screen_size.y * edge_width) / vertex_1[2];
            let radius_2 = (screen_size.y * edge_width) / vertex_2[2];
            
            let mut color_1 = color_from_wv(&vertex_1, w_scale, scene.edge_colors[i / 2]);
            color_1.a *= fade_from_depth(vertex_1[2], near, far, zoom);
            color_1.a *= 1.0 - (distance_from_nvolume(&vertex_1, 5) * w_scale).clamp(0.0, 1.0);
            
            let mut color_2 = color_from_wv(&vertex_2, w_scale, scene.edge_colors[i / 2]);
            color_2.a *= fade_from_depth(vertex_2[2], near, far, zoom);
            color_2.a *= 1.0 - (distance_from_nvolume(&vertex_2, 5) * w_scale).clamp(0.0, 1.0);
            
            draw_variable_width_line(project_vertex(&vertex_1, render_size, screen_size.clone()), project_vertex(&vertex_2, render_size, screen_size.clone()), radius_1 * render_size, radius_2 * render_size, color_1, color_2);
        }
        
    }
}
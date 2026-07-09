use macroquad::prelude::*;
use nalgebra::{DMatrix, DVector};

use crate::color::*;
use crate::math::*;
use crate::scene::*;

pub fn draw_variable_width_line(start_point: Vec2, end_point: Vec2, start_radius: f32, end_radius: f32, color: Color) {
    if color.a > 0.0 {
        let edge_direction = (end_point - start_point).normalize();
        let left_of_edge = vec2(edge_direction.y, -edge_direction.x);
        let right_of_edge = vec2(-edge_direction.y, edge_direction.x);
        
        draw_triangle(
            start_point + (left_of_edge * start_radius),
            start_point + (right_of_edge * start_radius),
            end_point + (left_of_edge * end_radius),
            color
        );
        
        draw_triangle(
            end_point + (left_of_edge * end_radius),
            end_point + (right_of_edge * end_radius),
            start_point + (right_of_edge * start_radius),
            color
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
            
            let edge_center = (&vertex_1 + &vertex_2) / 2.0;
            
            let mut color = color_from_wv(&edge_center, w_scale, scene.edge_colors[i / 2]);
            color.a *= fade_from_depth(edge_center[2], near, far, zoom);
            color.a *= 1.0 - (distance_from_nvolume(&edge_center, 5) * w_scale).clamp(0.0, 1.0);
            
            draw_variable_width_line(project_vertex(&vertex_1, render_size, screen_size.clone()), project_vertex(&vertex_2, render_size, screen_size.clone()), radius_1 * render_size, radius_2 * render_size, color);
        }
        
    }
}
use macroquad::audio::load_sound_from_bytes;
use macroquad::audio::play_sound_once;
use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::*;
use na::Vector2;
use nalgebra::VecStorage;
use nalgebra::{self as na, DMatrix, DVector};
use std;
use std::env;
use std::f32::consts::TAU;
use std::f32;
use std::usize;
use std::vec;

use crate::loader::*;
use crate::math::*;
use crate::scene::Scene;

mod scene;
mod loader;
mod math;

fn draw_variable_width_line(start_point: Vec2, end_point: Vec2, start_radius: f32, end_radius: f32, color: Color) {
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

fn mouse_control(previous_mouse_pos: Vector2<f32>, dimension: usize, shape_matrix: DMatrix<f32>, axis: usize, sensitivity: f32) -> DMatrix<f32> {
    if axis < dimension {
        return rotate_matrix(1, axis, (mouse_position().1 - previous_mouse_pos.y) * -sensitivity, dimension) * rotate_matrix(0, axis, (mouse_position().0 - previous_mouse_pos.x) * sensitivity, dimension) * shape_matrix;
    } else {
        return shape_matrix;
    }
}

fn color_from_hue(hue: f32) -> Color {
    let kr = f32::fract((5.0 + hue * 6.0) / 6.0) * 6.0;
    let kg = f32::fract((3.0 + hue * 6.0) / 6.0) * 6.0;
    let kb = f32::fract((1.0 + hue * 6.0) / 6.0) * 6.0;
    
    let r = 1.0 - f32::max(f32::min(f32::min(kr, 4.0 - kr), 1.0), 0.0);
    let g = 1.0 - f32::max(f32::min(f32::min(kg, 4.0 - kg), 1.0), 0.0);
    let b = 1.0 - f32::max(f32::min(f32::min(kb, 4.0 - kb), 1.0), 0.0);
    
    return Color::new(r, g, b, 1.0);
}

fn color_from_wv(vector: &DVector<f32>, w_scale: f32, edge_color: Color) -> Color {
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

fn fade_from_depth(z: f32, near: f32, far: f32, zoom: f32) -> f32 {
    1.0 - clamp(f32::inverse_lerp(near + zoom, far + zoom, z), 0.0, 1.0)
}

fn render(scene: &Scene, subdivisions: i32, shape_matrix: &DMatrix<f32>, shape_position: &DVector<f32>, edge_width: f32, near: f32, far: f32, zoom: f32, w_scale: f32, render_size: f32, screen_size: &Vec2) {
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

#[macroquad::main("nD Renderer")]
async fn main() {
    let args: Vec<String> = env::args().collect();

    const DONE_SOUND_BYTES: &[u8] = include_bytes!(".././done.wav");
    
    let mut scene = Scene::setup(&args);
    
    load_polytope(&mut scene);
    
    let mut shape_matrix = DMatrix::identity(scene.dimension, scene.dimension);
    let mut shape_position = DVector::zeros(scene.dimension);
    shape_position[2] = 2.0;
    
    // despite shape_matrix being defined the exact same way, only this variable needs to specify its type. ???
    let mut rotational_offset: nalgebra::Matrix<f32, nalgebra::Dyn, nalgebra::Dyn, VecStorage<f32, nalgebra::Dyn, nalgebra::Dyn>> = DMatrix::identity(scene.dimension, scene.dimension);
    
    let mut render_size= 0.5;
    let mut edge_width= 1.0 / 84.0;
    let mut zoom = 2.0;
    
    let mut w_scale: f32 = 0.5;
    let mut near = -1.0;
    let mut far = 0.5;
    
    let mut previous_mouse_pos = Vector2::new(0.0, 0.0);
    
    let mut subdivisions = 1;
	
	let facet_expansion_key_speed = f32::exp2(0.25); // 2 ^ 1/4
    
    let mut image_index = -2;
    
    let mut rotations: Vec<usize> = vec![];
    let mut rotations_global_vs_local: Vec<bool> = vec![];
    let mut rotation_amounts: Vec<f32> = vec![];
    
    let mut starting_position: Vec<f32> = vec![];
    let mut motion: Vec<f32> = vec![];
    
    let done_sound = load_sound_from_bytes(DONE_SOUND_BYTES).await.unwrap();
    
    let mut virtual_image = render_target(
        scene.resolution,
        scene.resolution,
    );
    
    set_window_size(1024, 1024);

    loop {
        // Rotate Shape
        
        if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Middle) {
            if is_key_down(KeyCode::LeftControl) {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 3, -1.0/216.0);
            } else if is_key_down(KeyCode::Z) {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 4, -1.0/216.0);
            } else if is_key_down(KeyCode::X) {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 5, -1.0/216.0);
            } else if is_key_down(KeyCode::C) {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 6, -1.0/216.0);
            } else if is_key_down(KeyCode::V) {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 7, -1.0/216.0);
            } else if is_key_down(KeyCode::B) {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 8, -1.0/216.0);
            } else if is_key_down(KeyCode::N) {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 9, -1.0/216.0);
            } else {
                shape_matrix = mouse_control(previous_mouse_pos, scene.dimension, shape_matrix, 2, 1.0/216.0);
            }
        }
        
        if is_mouse_button_down(MouseButton::Right) {
            let angle_diff = Vec2::new(mouse_position().0 - (screen_width() / 2.0), mouse_position().1 - (screen_height() / 2.0)).angle_between(Vec2::new(previous_mouse_pos.x - (screen_width() / 2.0), previous_mouse_pos.y - (screen_height() / 2.0)));
            
            shape_matrix = rotate_matrix(0, 1, angle_diff, scene.dimension) * &shape_matrix;
        }
        
        (previous_mouse_pos.x, previous_mouse_pos.y) = mouse_position();
        
        let scroll = mouse_wheel().1;
        if scroll < 0.0 {
            if is_key_down(KeyCode::LeftControl) {
                zoom *= 13.0/12.0;
                render_size *= 13.0/12.0;
            } else if is_key_down(KeyCode::LeftShift) {
                edge_width *= 12.0/13.0;
            } else {
                render_size *= 12.0/13.0;
            }
        } else if scroll > 0.0 {
            if is_key_down(KeyCode::LeftControl) {
                zoom *= 12.0/13.0;
                render_size *= 12.0/13.0;
            } else if is_key_down(KeyCode::LeftShift) {
                edge_width *= 13.0/12.0;
            } else {
                render_size *= 13.0/12.0;
            }
        }
        shape_position[2] = zoom;
        
        if is_key_down(KeyCode::Q) {
            near += get_frame_time();
        }
        if is_key_down(KeyCode::A) {
            near -= get_frame_time();
        }
        if is_key_down(KeyCode::W) {
            far += get_frame_time();
        }
        if is_key_down(KeyCode::S) {
            far -= get_frame_time();
        }
        if is_key_down(KeyCode::E) {
            w_scale *= 1.0 - get_frame_time();
        }
        if is_key_down(KeyCode::D) {
            w_scale *= 1.0 + get_frame_time();
        }
        if is_key_pressed(KeyCode::R) {
            subdivisions += 1;
        }
        if is_key_pressed(KeyCode::F) {
            subdivisions -= 1;
            if subdivisions == 0 {
                subdivisions = 1;
            }
        }
        if is_key_pressed(KeyCode::T) { // increases facet_expansion
            scene.clear_polytope();
			scene.facet_expansion = 1.0 - (1.0 - scene.facet_expansion) / facet_expansion_key_speed;
            load_polytope(&mut scene);
        }
		if is_key_pressed(KeyCode::G) { // decreases facet_expansion
            scene.clear_polytope();
			scene.facet_expansion = 1.0 - (1.0 - scene.facet_expansion) * facet_expansion_key_speed;
			if scene.facet_expansion < 1.0 - 1.0 / facet_expansion_key_speed {
				scene.facet_expansion = 0.0;
			}
            load_polytope(&mut scene);
        }
        if is_key_pressed(KeyCode::Y) {
            scene.clear_polytope();
			scene.facet_expansion_rank += 1;
			if scene.facet_expansion_rank > scene.dimension - 1 {
				scene.facet_expansion_rank = scene.dimension - 1;
			}
            load_polytope(&mut scene);
        }
		if is_key_pressed(KeyCode::H) {
            scene.clear_polytope();
			scene.facet_expansion_rank -= 1;
			if scene.facet_expansion_rank < 2 {
				scene.facet_expansion_rank = 2;
			}
            load_polytope(&mut scene);
        }
        if is_key_pressed(KeyCode::Key0) {
            scene = Scene::setup(&args);
            load_polytope(&mut scene);
            if scene.dimension != shape_position.nrows() {
                shape_matrix = DMatrix::identity(scene.dimension, scene.dimension);
                rotational_offset = DMatrix::identity(scene.dimension, scene.dimension);
                shape_position = DVector::zeros(scene.dimension);
            }
            virtual_image = render_target(scene.resolution, scene.resolution);
        }
        if is_key_pressed(KeyCode::Key1) {
            rotational_offset = shape_matrix.clone() * rotational_offset;
            shape_matrix = DMatrix::identity(scene.dimension, scene.dimension);
        }
        
        if image_index > -1 {
            // set camera to render target
            set_camera(&Camera2D {
                render_target: Some(virtual_image.clone()),
                zoom: vec2(1.0 / (scene.resolution as f32) * 2.0, 1.0 / (scene.resolution as f32) * -2.0),
                target: vec2((scene.resolution as f32) / 2.0, (scene.resolution as f32) / 2.0),
                ..Default::default()
            });
            
            // render the scene
            render(&scene, subdivisions, &(&shape_matrix * &rotational_offset), &shape_position, edge_width, near, far, zoom, w_scale, render_size, &vec2(scene.resolution_vector.x, scene.resolution_vector.y));
            
            // go back to the screen
            set_default_camera();
        }
        
        // render the scene to the screen
        render(&scene, subdivisions, &(&shape_matrix * &rotational_offset), &shape_position, edge_width, near, far, zoom, w_scale, render_size, &vec2(screen_width(), screen_height()));
        
        if image_index > -1 { // During the loop
            for i in (0..rotations.len()).step_by(2) {
                let rotation_matrix = rotate_matrix(rotations[i], rotations[i + 1], rotation_amounts[i / 2] / (scene.frame_count as f32), shape_matrix.ncols());
                if rotations_global_vs_local[i / 2] {
                    shape_matrix = &shape_matrix * rotation_matrix;
                } else {
                    shape_matrix = rotation_matrix * &shape_matrix;
                }
            }
            for i in 0..motion.len() {
                if i != 2 {
                    shape_position[i] += motion[i] / (scene.frame_count as f32);
                }
            }
            
            virtual_image.texture.get_texture_data().export_png(&format!("./images/{:03}.png", image_index));
            
            image_index += 1;
        }
        if image_index == -1 {
            image_index = 0;
        }
        
        if image_index == scene.frame_count { // End
            set_default_camera();
            image_index = -2;
            for i in 0..scene.dimension {
                shape_position[i] = 0.0;
            }
            
            play_sound_once(&done_sound);
        }
        
        if is_key_pressed(KeyCode::Escape) {
            set_default_camera();
            for i in 0..scene.dimension {
                shape_position[i] = 0.0;
            }
            for i in (0..rotations.len()).step_by(2) {
                let rotation_matrix = rotate_matrix(rotations[i], rotations[i + 1], (rotation_amounts[i / 2] / (scene.frame_count as f32)) * (-image_index) as f32, shape_matrix.ncols());
                if rotations_global_vs_local[i / 2] {
                    shape_matrix = &shape_matrix * rotation_matrix;
                } else {
                    shape_matrix = rotation_matrix * &shape_matrix;
                }
            }
            image_index = -2;
        }
        
        if is_key_pressed(KeyCode::Enter) { // Start
            image_index = -1;
            
            if !std::path::Path::new("./rotations.txt").exists() {
                panic!("no rotations.txt file!!!!");
            }
            if !std::path::Path::new("./motion.txt").exists() {
                panic!("no motion.txt file!!!!");
            }

            let rotation_file_contents = std::fs::read_to_string("./rotations.txt").unwrap();
            
            rotations.clear();
            rotations_global_vs_local.clear();
            rotation_amounts.clear();
            
            let mut rotation_file_values: Vec<usize> = vec![];
            
            for line in rotation_file_contents.lines() {
                let mut value_count = 0;
                
                // go through the line of text to find the numbers
                for number_string in line.split(" ") {
                    let number: usize = number_string.parse().unwrap();
                    
                    rotation_file_values.push(number);
                    value_count += 1;
                }
                
                if value_count == 2 {
                    rotation_file_values.push(0);
                }
                if value_count == 3 {
                    rotation_file_values.push(1);
                }
            }
            
            for i in (0..rotation_file_values.len()).step_by(4) {
                rotations.push(rotation_file_values[i]);
                rotations.push(rotation_file_values[i + 1]);
                
                rotations_global_vs_local.push(rotation_file_values[i + 2] == 1);
                
                rotation_amounts.push(TAU / (rotation_file_values[i + 3] as f32));
            }
            
            let motion_file_contents = std::fs::read_to_string("./motion.txt").unwrap();
            
            starting_position.clear();
            motion.clear();
            
            let mut index = 0;
            for line in motion_file_contents.lines() {
                // go through the line of text to find the numbers
                for number_string in line.split(" ") {
                    let number: f32 = number_string.parse().unwrap();
                    
                    if index == 0 {
                        starting_position.push(number);
                    } else if index == 1 {
                        motion.push(number);
                    }
                }
                
                index += 1;
            }
            
            for i in 0..starting_position.len() {
                if i != 2 {
                    shape_position[i] = starting_position[i];
                }
            }
        }
        
        next_frame().await
    }
}

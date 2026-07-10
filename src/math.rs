use macroquad::math::Vec2;
use nalgebra::{DMatrix, DVector};

pub fn rotate_matrix(axis_1: usize, axis_2: usize, angle_in_radians: f32, dimension: usize) -> DMatrix<f32> {
    let mut matrix = DMatrix::identity(dimension, dimension);
    
    matrix[axis_1 + (axis_1 * dimension)] = f32::cos(angle_in_radians);
    matrix[axis_2 + (axis_1 * dimension)] = f32::sin(angle_in_radians);
    
    matrix[axis_1 + (axis_2 * dimension)] = -f32::sin(angle_in_radians);
    matrix[axis_2 + (axis_2 * dimension)] = f32::cos(angle_in_radians);
    
    return matrix;
}

pub fn project_vertex(vertex: &DVector<f32>, render_size: f32, screen_size: Vec2) -> Vec2 {
    let mut screen_vertex = Vec2::new(-vertex[0], vertex[1]) / (vertex[2]);
    screen_vertex *= -screen_size.y * render_size;
    screen_vertex += screen_size / 2.0;
    
    screen_vertex
}

pub fn distance_from_nvolume(vertex: &DVector<f32>, n: usize) -> f32 {
    if vertex.len() < n {
        return 0.0;
    }
    
    let mut distance: f32 = 0.0;
    for axis in 0..(vertex.len() - n) {
        distance += vertex[axis + n] * vertex[axis + n];
    }
    
    f32::sqrt(distance)
}
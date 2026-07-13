use std::path::PathBuf;

use macroquad::{color::*, rand::ChooseRandom};
use nalgebra::DVector;
use walkdir::WalkDir;

use crate::{Scene};

fn get_vertices_from_element(polytope_data: &Vec<Vec<Vec<usize>>>, element_vertices: &mut Vec<usize>, element_edges: &mut Vec<usize>, rank: usize, index: usize) {
    // loop through the facet
    let mut i = 0;
    for sub_element in &polytope_data[rank - 2][index] {
        if rank == 2 { // Faces, add vertices
            element_vertices.push(*sub_element);
            
            // Add the edge, which will be a pair of two integers, each pointing to a vertex ID in the global polytope.
            element_edges.push(*sub_element);
            
            // Get the next vertex ID ID, and append it. Remember that polygons are circular.
            let next_vertex_id_id = (i + 1) % polytope_data[rank - 2][index].len();
            element_edges.push(polytope_data[rank - 2][index][next_vertex_id_id]);
        } else { // Non faces, check sub elements
            let mut sub_vertices: Vec<usize> = vec![];
            let mut sub_edges: Vec<usize> = vec![];
            
            get_vertices_from_element(polytope_data, &mut sub_vertices, &mut sub_edges, rank - 1, *sub_element);
            
            // Merge faces and other elements correctly
            for vertex in sub_vertices.iter() {
                if !element_vertices.contains(vertex) {
                    element_vertices.push(vertex.clone());
                }
            }
            for edge in (0..sub_edges.len()).step_by(2) {
                let vertex_index_a = sub_edges[edge];
                let vertex_index_b = sub_edges[edge + 1];
                
                let mut found_duplicate = false;
                for edge_start_index in (0..element_edges.len()).step_by(2) {
                    let edge_start = element_edges[edge_start_index];
                    let edge_end = element_edges[edge_start_index + 1];
                    
                    if (vertex_index_a == edge_start && vertex_index_b == edge_end) || (vertex_index_a == edge_end && vertex_index_b == edge_start) {
                        found_duplicate = true;
                        break;
                    }
                }
                
                if !found_duplicate {
                    element_edges.push(vertex_index_a);
                    element_edges.push(vertex_index_b);
                }
            }
        }
        
        i += 1;
    }
}

pub fn load_polytope(scene: &mut Scene, rand: bool) {
    if rand {
        let files: Vec<PathBuf> = WalkDir::new(scene.polytopes_folder.as_str())
            .into_iter()
            .filter_map(|entry| entry.ok())      // Ignore unreadable files/directories
            .filter(|entry| entry.file_type().is_file()) // Filter out directories
            .map(|entry| entry.into_path())      // Convert WalkDir Entry to PathBuf
            .collect();
        println!("Found {} files: {:?}", files.len(), files);

        let file = files.choose().expect("File cannot be found or doesnt exist!!!!");

        scene.polytope_path = file.display().to_string();

        println!("Choose: {}", scene.polytope_path);
    }

    let contents: String = std::fs::read_to_string(scene.polytope_path.as_str()).expect("File cannot be found or doesnt exist!!!!");
    
    let mut state: u8 = 0;
    
    let mut rank: u8 = 0;
    
    let mut full_lines_seen = 0;
    
    // vertices
    let mut polytope_vertices: Vec<DVector<f32>> = vec![];
    // rank, element, indices referencing previous rank
    let mut polytope_data: Vec<Vec<Vec<usize>>> = vec![];
    
    for line in contents.lines() {
        if line.starts_with("#") {
            continue;
        }
        
        if line.is_empty() {
            if state == 1 { // If done reading rank, start reading vertices
                if full_lines_seen == 1 {
                    state = 2;
                }
            } else if state == 2 { // If done reading vertices, start reading edges (faces)
                state = 3;
                polytope_data.push(vec![]);
            } else if state > 2 { // If done reading edges (faces), continue or stop depending on facet_expansion
                if scene.facet_expansion == 0.0 {
                    break;
                } else {
                    state += 1;
                    if state > rank {
                        break;
                    } else {
                        polytope_data.push(vec![]);
                    }
                }
            }
            
            continue;
        }
        
        if line.ends_with("OFF") {
            if line == "OFF" {
                scene.dimension = 3; // some dumbasses think that not having a number for 3D is okay, well, it's NOT,
                // it means I have to take time out of MY day to add an edge case for it every single time I
                // make an OFF importer. AGH.
            } else {
                scene.dimension = line[.. line.len() - 3].parse().unwrap();
            }
            
            rank = scene.dimension as u8;
            
            if scene.dimension < scene.min_dimension {
                scene.dimension = scene.min_dimension;
            }
            
            state = 1;
            continue;
        }
		
		if scene.facet_expansion_rank > usize::MAX / 2 { // relative value
			scene.facet_expansion_rank = (rank as usize).wrapping_add(usize::MAX).wrapping_sub(scene.facet_expansion_rank) + 1; // just let me use integer overflow >:(
		}
		if scene.facet_expansion_rank > rank as usize - 1 {
			scene.facet_expansion_rank = rank as usize - 1;
		}
		if scene.facet_expansion_rank < 2 {
			scene.facet_expansion_rank = 2;
		}
		if rank == 2 && scene.facet_expansion > 0.0 {
			scene.facet_expansion = 0.0;
		}
        
        full_lines_seen += 1;
        
        // Vertices
        if state == 2 {
            let mut vertex: Vec<f32> = vec![];
            
            for coordinate in line.split(" ") {
                if !coordinate.is_empty() {
                    vertex.push(coordinate.parse().unwrap());
                }
            }
            
            while vertex.len() < scene.min_dimension {
                vertex.push(0.0);
            }
            
            polytope_vertices.push(DVector::from_vec(vertex));
        }
        
        // Edges (actually faces)
        if state == 3 {
            // stores the vertex indices of the face
            let mut face: Vec<usize> = vec![];
            
            // go through the line of text to find the indices
            let mut index = 0;
            for number_string in line.split(" ") {
                let number: usize = number_string.parse().unwrap();
                
                // the first one is the size of the face. who needs that? I have .len() and I'm not afraid to use it.
                if index != 0 {
                    face.push(number);
                }
                
                index += 1;
            }
            
            polytope_data[0].push(face.clone());
            
            if scene.facet_expansion == 0.0 {
                // loop through the face to get all the edges
                for index in 0..face.len() {
                    let vertex_index_a = face[index];
                    let vertex_index_b = face[(index + 1) % face.len()];
                    
                    // make sure the edge or its opposite aren't in the edges array
                    let mut found_duplicate = false;
                    for edge_start_index in (0..scene.edges.len()).step_by(2) {
                        let edge_start = scene.edges[edge_start_index];
                        let edge_end = scene.edges[edge_start_index + 1];
                        
                        if (vertex_index_a == edge_start && vertex_index_b == edge_end) || (vertex_index_a == edge_end && vertex_index_b == edge_start) {
                            found_duplicate = true;
                            break;
                        }
                    }
                    
                    // add them
                    if !found_duplicate {
                        scene.edges.push(vertex_index_a);
                        scene.edges.push(vertex_index_b);
                        scene.edge_colors.push(WHITE);
                    }
                }
            }
        }
        
        if state > 3 {
            // stores the rank n-1 element indices of the rank n element
            let mut element: Vec<usize> = vec![];
            
            // go through the line of text to find the indices
            let mut index = 0;
            for number_string in line.split(" ") {
                let number: usize = number_string.parse().unwrap();
                
                // the first one is the size of the element. who needs that? I have .len() and I'm not afraid to use it.
                if index != 0 {
                    element.push(number);
                }
                
                index += 1;
            }
            
            polytope_data[(state - 3) as usize].push(element);
        }
    }
    
    // we now have the polytope_data.
    // polytope_data stores the faces, then the cells, tera, etc. Its length is 2 less than the rank.
    if scene.facet_expansion > 0.0 {
        // okay, we need to append vertices of every facet, scaled inward towards the average, to the polytope.
        for facet in 0..polytope_data[scene.facet_expansion_rank - 2].len() {
            let mut facet_vertices: Vec<usize> = vec![];
            let mut facet_edges: Vec<usize> = vec![];
            
            get_vertices_from_element(&polytope_data, &mut facet_vertices, &mut facet_edges, scene.facet_expansion_rank as usize, facet);
            
            // once that is done, loop over all the facet_vertices to determine the center.
            let mut facet_center: DVector<f32> = DVector::zeros(scene.dimension);
            for vertex in facet_vertices.iter() {
                facet_center += &polytope_vertices[*vertex];
            }
            facet_center /= facet_vertices.len() as f32;
            
            // vertices is the vertices of the mesh
            // polytope_vertices is the vertices of the polytope
            // fucking dumbass (for context, this used to say polytope_vertices.len(), and I struggled to find why it wasn't working)
            let past_vertex_count = scene.vertices.len();
            
            // loop over them again, subtracting each one by the center, multiplying by facet_expansion, and then adding the center
            for vertex in facet_vertices.iter() {
                scene.vertices.push(((&polytope_vertices[*vertex] - &facet_center) * scene.facet_expansion) + &facet_center);
            }
            
            for edge in facet_edges.iter() {
                // These variables are very poorly named, so I will explain
                // past_vertex_count is the number of vertices before a facet, so that's our starting point
                // we have edges as references to vertex IDs in the global polytope
                // but we want the edges as reference to vertex IDs in the facet
                // luckily we can just search the facet_vertices array for these IDs and then it'll work
                
                // edge is the global polytope vertex ID to the first or second half of an edge
                // facet_vertices contains the vertex IDs of the facet in relation to the global polytope
                
                scene.edges.push(past_vertex_count + facet_vertices.iter().position(|x| *x == *edge).unwrap());
            }
            
            for _i in 0..facet_edges.len()/2 {
                if rank == 3 {
                    match facet_vertices.len() {
                        3 => {
                            // A2, red
                            scene.edge_colors.push(Color { r: 213.0/255.0, g: 56.0/255.0, b: 56.0/255.0, a: 1.0 });
                        }
                        6 => {
                            // G2/2, light red
                            scene.edge_colors.push(Color { r: 208.0/255.0, g: 100.0/255.0, b: 100.0/255.0, a: 1.0 });
                        }
                        4 => {
                            // lies roughly on the axes
                            if (f32::abs(facet_center[0]) < 0.01 && f32::abs(facet_center[1]) < 0.01) ||
                            (f32::abs(facet_center[0]) < 0.01 && f32::abs(facet_center[2]) < 0.01) ||
                            (f32::abs(facet_center[2]) < 0.01 && f32::abs(facet_center[1]) < 0.01) {
                                // exclude H3
                                if polytope_data[0].len() < 30 {
                                    // B2, blue
                                    scene.edge_colors.push(Color { r: 43.0/255.0, g: 38.0/255.0, b: 135.0/255.0, a: 1.0 });
                                    continue;
                                }
                            }
                            
                            // K2, yellow
                            scene.edge_colors.push(Color { r: 229.0/255.0, g: 188.0/255.0, b: 38.0/255.0, a: 1.0 });
                        }
                        8 => {
                            // I2(8)/2, light blue
                            scene.edge_colors.push(Color { r: 87.0/255.0, g: 83.0/255.0, b: 153.0/255.0, a: 1.0 });
                        }
                        5 => {
                            // H2, purple
                            scene.edge_colors.push(Color { r: 139.0/255.0, g: 58.0/255.0, b: 177.0/255.0, a: 1.0 });
                            // green
                            // scene.edge_colors.push(Color { r: 66.0/255.0, g: 210.0/255.0, b: 58.0/255.0, a: 1.0 });
                        }
                        10 => {
                            // I2(10)/2, light purple
                            scene.edge_colors.push(Color { r: 147.0/255.0, g: 98.0/255.0, b: 170.0/255.0, a: 1.0 });
                            // light green
                            // scene.edge_colors.push(Color { r: 86.0/255.0, g: 220.0/255.0, b: 129.0/255.0, a: 1.0 });
                        }
                        _ => {
                            // ???
                            scene.edge_colors.push(MAGENTA);
                        }
                    }
                } else {
                    scene.edge_colors.push(WHITE);
                }
            }
        }
        
    } else {
        scene.vertices = polytope_vertices.clone();
    }
}

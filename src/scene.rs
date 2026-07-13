use macroquad::color::Color;
use nalgebra::{DVector, Vector2};

pub struct Scene {
    pub polytopes_folder: String,
    pub polytope_path: String,
    pub resolution: u32,
    pub frame_count: i32,
    pub facet_expansion: f32,
	pub facet_expansion_rank: usize,
    pub min_dimension: usize,
    pub dimension: usize,
    pub vertices: Vec<DVector<f32>>,
    pub edges: Vec<usize>,
    pub edge_colors: Vec<Color>,
    pub resolution_vector: Vector2<f32>,
}

impl Scene {
    pub fn setup(args: &Vec<String>) -> Self {
        if !std::path::Path::new("./setup.txt").exists() {
            panic!("no setup.txt file!!!!");
        }
        
        let setup_file_contents = std::fs::read_to_string("./setup.txt").unwrap();
        let lines: Vec<&str> = setup_file_contents.lines().collect();

		let polytope_path_pre: String;
		if args.len() < 2 {
			polytope_path_pre = lines[1].to_string();
		}
		else {
			polytope_path_pre = args[1].clone();
		}
		
        Scene {
            polytopes_folder: lines[0].to_string(),
			polytope_path: polytope_path_pre,
            resolution: lines[2].parse().unwrap(),
            frame_count: lines[3].parse().unwrap(),
            min_dimension: lines[4].parse().unwrap(),
            facet_expansion: lines[5].parse().unwrap(),
			facet_expansion_rank: lines[6].parse::<isize>().unwrap() as usize, // converts negative values to super high (integer underflow) ones. necessary for relative to rank values
            dimension: 0,
            vertices: vec![],
            edges: vec![],
            edge_colors: vec![],
            resolution_vector: Vector2::new(lines[2].parse().unwrap(), lines[2].parse().unwrap())
        }
    }
	
	pub fn clear_polytope(&mut self) {
		self.vertices.clear();
        self.edges.clear();
	}
}
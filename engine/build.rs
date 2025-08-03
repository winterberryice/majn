use image::{GenericImage, RgbaImage};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("terrain_atlas.png");

    let texture_atlas_json_path = Path::new("assets/textures/texture_atlas.json");
    let json_content = fs::read_to_string(texture_atlas_json_path)
        .expect("Failed to read texture_atlas.json");
    let texture_matrix: Vec<Vec<String>> =
        serde_json::from_str(&json_content).expect("Failed to parse texture_atlas.json");

    const ATLAS_COLS: u32 = 16;
    let atlas_rows = texture_matrix.len() as u32;

    let atlas_width = ATLAS_COLS * 16;
    let atlas_height = atlas_rows * 16;

    let mut atlas = RgbaImage::new(atlas_width, atlas_height);

    for (row_idx, row) in texture_matrix.iter().enumerate() {
        for (col_idx, file_name) in row.iter().enumerate() {
            if col_idx >= ATLAS_COLS as usize {
                panic!(
                    "Row {} in texture_atlas.json has more than {} textures.",
                    row_idx, ATLAS_COLS
                );
            }

            let texture_path = Path::new("assets/textures").join(file_name);
            let img = image::open(&texture_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to open image {}: {}",
                    texture_path.display(),
                    e
                )
            });

            atlas
                .copy_from(&img, col_idx as u32 * 16, row_idx as u32 * 16)
                .expect("Failed to copy texture to atlas");
        }
    }

    atlas.save(&dest_path).expect("Failed to save atlas");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/textures/texture_atlas.json");
    println!("cargo:rerun-if-changed=assets/textures/block/");
}

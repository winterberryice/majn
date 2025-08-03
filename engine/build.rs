use image::{GenericImage, RgbaImage};
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("terrain_atlas.png");
    let textures_dir = Path::new("assets/textures/block");

    let texture_files = [
        "grass_block_top.png",
        "grass_block_side.png",
        "dirt.png",
        "bedrock.png",
        "oak_log.png",
        "oak_log_top.png",
        "oak_leaves.png",
    ];

    let mut images = Vec::new();
    for file_name in &texture_files {
        let path = textures_dir.join(file_name);
        images.push(image::open(&path).unwrap());
    }

    // All textures are 16x16, so we can create a 3x3 atlas
    let atlas_width = 16 * 3;
    let atlas_height = 16 * 3;
    let mut atlas = RgbaImage::new(atlas_width, atlas_height);

    for (i, img) in images.iter().enumerate() {
        let col = (i % 3) as u32;
        let row = (i / 3) as u32;
        atlas
            .copy_from(img, col * 16, row * 16)
            .unwrap();
    }

    atlas.save(&dest_path).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/textures/block/");
}

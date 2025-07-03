use glam::Vec3;
use std::time::Instant;
use wgpu::TextureFormat;
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{Extra, OwnedSection, OwnedText, ab_glyph::FontArc},
};
// NEW: We need to know about the Block struct to get its info.
use crate::block::Block;

const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/Roboto-Regular.ttf");

pub struct DebugOverlay {
    brush: TextBrush,
    section: OwnedSection<Extra>,
    visible: bool,
    last_frame_time: Instant,
    frame_count: u32,
    accumulated_time: f32,
    fps: u32,
    // NEW: Add a field to store info about the selected block.
    selected_block_info: Option<String>,
}

impl DebugOverlay {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let font = FontArc::try_from_slice(FONT_BYTES).expect(
            "Failed to load font. Make sure Roboto-Regular.ttf is in the correct location.",
        );
        let brush = BrushBuilder::using_font(font.clone())
            .with_depth_stencil(Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }))
            .build(device, config.width, config.height, config.format);

        let section = OwnedSection::default()
            .add_text(
                OwnedText::new("")
                    .with_scale(20.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            )
            .with_screen_position((10.0, 10.0))
            .with_bounds((config.width as f32, config.height as f32));

        Self {
            brush,
            section,
            visible: true,
            last_frame_time: Instant::now(),
            frame_count: 0,
            accumulated_time: 0.0,
            fps: 0,
            // NEW: Initialize the new field.
            selected_block_info: None,
        }
    }

    // NEW: A method to update the overlay with the selected block's data.
    // We'll call this from main.rs every frame.
    pub fn update_selection_info(&mut self, block: Option<&Block>) {
        if let Some(b) = block {
            self.selected_block_info = Some(format!(
                "Block: {:?}\nSunlight: {}\nBlocklight: {}",
                b.block_type, b.sun_light, b.block_light
            ));
        } else {
            self.selected_block_info = None;
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn update(&mut self, player_position: Vec3) {
        if !self.visible {
            self.last_frame_time = Instant::now();
            self.frame_count = 0;
            self.accumulated_time = 0.0;
            return;
        }

        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        self.accumulated_time += delta_time;
        self.frame_count += 1;

        if self.accumulated_time >= 1.0 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.accumulated_time -= 1.0;
        }

        // Start with the basic info.
        let mut text_content = format!(
            "FPS: {}\nPosition: {:.2}, {:.2}, {:.2}",
            self.fps, player_position.x, player_position.y, player_position.z
        );

        // NEW: Append the selected block info if it exists.
        if let Some(info) = &self.selected_block_info {
            text_content.push_str("\n---\n");
            text_content.push_str(info);
        }

        self.section.text = vec![
            OwnedText::new(text_content)
                .with_scale(20.0)
                .with_color([1.0, 1.0, 1.0, 1.0]),
        ];
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), wgpu_text::BrushError> {
        if self.visible {
            let borrowed_section = self.section.to_borrowed();
            self.brush
                .queue(device, queue, std::slice::from_ref(&borrowed_section))?;
        }
        Ok(())
    }

    pub fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        if self.visible {
            self.brush.draw(render_pass);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        self.brush.resize_view(width as f32, height as f32, queue);
        self.section.bounds = (width as f32, height as f32);
    }
}

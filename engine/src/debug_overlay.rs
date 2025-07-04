// Corrected use statement based on glyph_brush re-exports
use crate::block::Block;
use glam::Vec3;
use std::time::Instant;
use wgpu::TextureFormat; // Import TextureFormat
use wgpu_text::{
    BrushBuilder,
    TextBrush,
    glyph_brush::{Extra, OwnedSection, OwnedText, ab_glyph::FontArc}, // Import Section and Text from glyph_brush
};

// This line assumes a font file named "Roboto-Regular.ttf" is in your assets/fonts/ directory
// or configured in your project's root. Make sure the path is correct for your setup.
const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/Roboto-Regular.ttf");

pub struct DebugOverlay {
    brush: TextBrush,
    // The type is now just 'Section', but we imported it from glyph_brush
    section: OwnedSection<Extra>,
    visible: bool,
    last_frame_time: Instant,
    frame_count: u32,
    accumulated_time: f32,
    fps: u32,
}

impl DebugOverlay {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let font = FontArc::try_from_slice(FONT_BYTES).expect(
            "Failed to load font. Make sure Roboto-Regular.ttf is in the correct location.",
        );
        let brush = BrushBuilder::using_font(font.clone())
            .with_depth_stencil(Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true, // Text should probably write to depth buffer
                depth_compare: wgpu::CompareFunction::Less, // Match main pipeline's comparison
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }))
            .build(device, config.width, config.height, config.format);

        // We don't need to prefix with a module name here because we `use`d them directly
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
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    // pub fn is_visible(&self) -> bool {
    //     self.visible
    // }

    pub fn update(&mut self, player_position: Vec3, selected_block: Option<&Block>) {
        if !self.visible {
            // Reset FPS calculation when not visible to avoid large delta_time on re-enabling
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

        // Update FPS counter once per second
        if self.accumulated_time >= 1.0 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.accumulated_time -= 1.0;
        }

        let mut text_content = format!(
            "FPS: {}\nPosition: {:.2}, {:.2}, {:.2}",
            self.fps, player_position.x, player_position.y, player_position.z
        );

        if let Some(block) = selected_block {
            text_content.push_str(&format!(
                "\n\nSelected Block: type: {:?}, sky_light: {}, block_light: {}",
                block.block_type, block.sky_light, block.block_light
            ));
        }

        // Update the section's text.
        self.section.text = vec![
            OwnedText::new(text_content)
                .with_scale(20.0)
                .with_color([1.0, 1.0, 1.0, 1.0]),
        ];
    }

    // This function prepares the text for rendering
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), wgpu_text::BrushError> {
        if self.visible {
            // Queue the section for drawing. The brush needs a slice of sections.
            let borrowed_section = self.section.to_borrowed();
            self.brush
                .queue(device, queue, std::slice::from_ref(&borrowed_section))?;
        }
        Ok(())
    }

    // This function performs the actual drawing. It must be called within a RenderPass.
    pub fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        if self.visible {
            self.brush.draw(render_pass);
        }
    }

    // Call this when the window is resized
    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        // Update the brush's view projection
        self.brush.resize_view(width as f32, height as f32, queue);
        // Update the text section's bounds
        self.section.bounds = (width as f32, height as f32);
    }
}

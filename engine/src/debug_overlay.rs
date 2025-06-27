use wgpu_text::{
    glyph_brush::ab_glyph::FontArc,
    section, // Import the 'section' module
    BrushBuilder,
    TextBrush
};
use std::time::Instant;
use glam::Vec3;

const FONT_BYTES: &[u8] = include_bytes!("Roboto-Regular.ttf"); // Assuming Roboto-Regular.ttf is next to this file

pub struct DebugOverlay {
    brush: TextBrush,
    section: section::Section<'static>, // Use section::Section
    visible: bool,
    last_frame_time: Instant,
    frame_count: u32,
    accumulated_time: f32,
    fps: u32,
    font: FontArc,
}

impl DebugOverlay {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let font = FontArc::try_from_slice(FONT_BYTES).expect("Failed to load font");
        let brush = BrushBuilder::using_font(font.clone())
            .build(device, config.width, config.height, config.format);

        // Use section::Section and section::Text
        let section = section::Section::default()
            .add_text(section::Text::new("").with_scale(20.0).with_color([1.0, 1.0, 1.0, 1.0]))
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
            font,
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
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

        let text_content = format!(
            "FPS: {}\nPosition: {:.2}, {:.2}, {:.2}",
            self.fps,
            player_position.x,
            player_position.y,
            player_position.z
        );

        self.section.text = vec![section::Text::new(&text_content) // Use section::Text
            .with_scale(20.0)
            .with_color([1.0, 1.0, 1.0, 1.0])];
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), wgpu_text::BrushError>{
        if self.visible {
            // Ensure the section is passed as a slice of references
            self.brush.queue(device, queue, vec![&self.section])?;
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

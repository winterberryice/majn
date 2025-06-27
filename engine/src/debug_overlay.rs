// Use wgpu_text types directly, not through a 'section' module
use wgpu_text::{
    glyph_brush::{ab_glyph::FontArc, Section, Text}, // Corrected import path
    BrushBuilder,
    TextBrush,
};
use std::time::Instant;
use glam::Vec3;

// This line assumes a font file named "Roboto-Regular.ttf" is in your src/ directory
// or configured in your project's root. Make sure the path is correct for your setup.
const FONT_BYTES: &[u8] = include_bytes!("Roboto-Regular.ttf");

pub struct DebugOverlay {
    brush: TextBrush,
    // The type is now just 'Section', but we imported it from glyph_brush
    section: Section<'static>,
    visible: bool,
    last_frame_time: Instant,
    frame_count: u32,
    accumulated_time: f32,
    fps: u32,
    font: FontArc,
}

impl DebugOverlay {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let font = FontArc::try_from_slice(FONT_BYTES).expect("Failed to load font. Make sure Roboto-Regular.ttf is in the correct location.");
        let brush = BrushBuilder::using_font(font.clone())
            .build(device, config.width, config.height, config.format);

        // We don't need to prefix with a module name here because we `use`d them directly
        let section = Section::default()
            .add_text(Text::new("").with_scale(20.0).with_color([1.0, 1.0, 1.0, 1.0]))
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

        let text_content = format!(
            "FPS: {}\nPosition: {:.2}, {:.2}, {:.2}",
            self.fps,
            player_position.x,
            player_position.y,
            player_position.z
        );

        // THE FIX IS HERE: Remove the '&' to move ownership of the String
        // into the Text object, giving it a 'static lifetime.
        self.section.text = vec![Text::new(text_content)
            .with_scale(20.0)
            .with_color([1.0, 1.0, 1.0, 1.0])];
    }

    // This function prepares the text for rendering
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), wgpu_text::BrushError>{
        if self.visible {
            // Queue the section for drawing. The brush needs a slice of sections.
            self.brush.queue(device, queue, &[&self.section])?;
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

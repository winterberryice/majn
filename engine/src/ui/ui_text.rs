use wgpu_text::{
    BrushBuilder,
    TextBrush,
    glyph_brush::{OwnedSection, ab_glyph::FontArc},
};

const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/Roboto-Regular.ttf");

pub struct UIText {
    brush: TextBrush,
}

impl UIText {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let font = FontArc::try_from_slice(FONT_BYTES).expect("Failed to load font");
        let brush = BrushBuilder::using_font(font).build(
            device,
            config.width,
            config.height,
            config.format,
        );

        Self { brush }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sections: &[OwnedSection],
    ) -> Result<(), wgpu_text::BrushError> {
        let sections_borrowed: Vec<_> = sections.iter().map(|s| s.to_borrowed()).collect();
        self.brush.queue(device, queue, &sections_borrowed)?;
        Ok(())
    }

    pub fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        self.brush.draw(render_pass);
    }

    pub fn resize(&mut self, width: u32, height: u32, queue: &wgpu::Queue) {
        self.brush.resize_view(width as f32, height as f32, queue);
    }
}

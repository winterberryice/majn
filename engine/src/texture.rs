use image::GenericImageView;
// Unused: ImageError, NonZeroU32
use std::path::Path;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
    ) -> Result<Self, String> {
        let path_buf = path.as_ref().to_path_buf();
        let label = path_buf.file_name().unwrap_or_default().to_str();

        // Load the image from disk
        let img = image::open(&path_buf).map_err(|e| {
            format!(
                "Failed to open image {}: {}",
                path_buf.display(),
                e.to_string()
            )
        })?;
        let dimensions = img.dimensions(); // dimensions are (usize, usize)
        let rgba = img.to_rgba8(); // Convert to RGBA8

        let size = wgpu::Extent3d {
            width: dimensions.0 as u32, // Cast to u32
            height: dimensions.1 as u32, // Cast to u32
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // Use sRGB for color textures
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture { // Struct name seems correct for wgpu 0.19-0.20 / 25.0.x
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout { // Struct name seems correct for wgpu 0.19-0.20 / 25.0.x
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0 as u32), // Cast to u32
                rows_per_image: Some(dimensions.1 as u32),   // Cast to u32
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    // Helper to create a placeholder texture if loading fails or for testing
    pub fn create_placeholder(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
    ) -> Self {
        let dimensions = (16u32, 16u32); // Standard missing texture size, ensure u32
        let mut rgba_data = vec![0u8; (4 * dimensions.0 * dimensions.1) as usize];
        // Create a checkerboard pattern (purple and black)
        for y in 0..dimensions.1 { // dimensions.1 is u32
            for x in 0..dimensions.0 { // dimensions.0 is u32
                let idx = ((y * dimensions.0 + x) * 4) as usize; // Calculate index as usize
                if (x / 4 + y / 4) % 2 == 0 {
                    rgba_data[idx] = 255; // R
                    rgba_data[idx + 1] = 0;   // G
                    rgba_data[idx + 2] = 255; // B
                    rgba_data[idx + 3] = 255; // A
                } else {
                    rgba_data[idx] = 0;   // R
                    rgba_data[idx + 1] = 0;   // G
                    rgba_data[idx + 2] = 0;   // B
                    rgba_data[idx + 3] = 255; // A
                }
            }
        }

        let size = wgpu::Extent3d {
            width: dimensions.0, // Already u32
            height: dimensions.1, // Already u32
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture { // Struct name is correct
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba_data,
            wgpu::ImageDataLayout { // Struct name is correct
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0), // dimensions.0 is u32
                rows_per_image: Some(dimensions.1),   // dimensions.1 is u32
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

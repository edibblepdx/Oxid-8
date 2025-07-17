use anyhow::*;
use oxid8_core::{SCREEN_AREA, SCREEN_HEIGHT, SCREEN_WIDTH};

const WHITE: [u8; 4] = [255, 255, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    size: wgpu::Extent3d,
}

impl Texture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[bool]) -> Result<Self> {
        let size = wgpu::Extent3d {
            width: SCREEN_WIDTH as u32,
            height: SCREEN_HEIGHT as u32,
            depth_or_array_layers: 1,
        };
        #[rustfmt::skip]
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let tx = Texture {
            texture,
            view,
            sampler,
            size,
        };
        tx.update(queue, bytes);

        Ok(tx)
    }

    pub fn update(&self, queue: &wgpu::Queue, screen: &[bool]) {
        let mut tx: Vec<u8> = vec![];
        tx.reserve(4 * SCREEN_AREA);

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                tx.extend_from_slice(if screen[x + y * SCREEN_WIDTH] {
                    &WHITE
                } else {
                    &BLACK
                });
            }
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &tx,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * SCREEN_WIDTH as u32),
                rows_per_image: Some(SCREEN_HEIGHT as u32),
            },
            self.size,
        );
    }
}

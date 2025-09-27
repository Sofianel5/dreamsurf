use std::{f32::consts::PI, fs, path::Path};

use anyhow::{Context, Result};
use bevy::math::Vec3;
use generative::{
    ImageData, ImageGenerationRequest, ImageGenerator, ImageOutputFormat, OpenAiImageGenerator,
};
use image::{RgbaImage, imageops, load_from_memory};
use ktx2_rw::{Ktx2Texture, VkFormat};
use tokio::runtime::Builder;

const GENERATED_TEXTURE_PATH: &str = "cubemaps/generated/sky.ktx2";

fn convert_to_ktx2(bytes: &[u8]) -> Result<Ktx2Texture> {
    let mut image = load_from_memory(bytes)
        .context("failed to decode image from downloaded bytes")?
        .to_rgba8();

    let width = image.width();
    let original_height = image.height();
    let target_height = (width / 2).max(1);

    if original_height > target_height {
        tracing::debug!(
            width,
            original_height,
            target_height,
            "cropping sky panorama to maintain 2:1 aspect"
        );
        let cropped = imageops::crop(&mut image, 0, 0, width, target_height).to_image();
        image = cropped;
    } else if original_height < target_height {
        tracing::warn!(
            width,
            original_height,
            target_height,
            "sky panorama shorter than 2:1; conversion may stretch"
        );
    }

    let width = image.width();
    let height = image.height();
    let face_size = height;
    let mut texture =
        Ktx2Texture::create(face_size, face_size, 1, 1, 6, 1, VkFormat::R8G8B8A8Unorm)
            .context("failed to create Ktx2Texture")?;

    let mut face_pixels = vec![0u8; (face_size * face_size * 4) as usize];

    for face in 0..6 {
        fill_cubemap_face(&image, face, face_size, &mut face_pixels);
        texture
            .set_image_data(0, 0, face, &face_pixels)
            .context("failed to set image data on Ktx2Texture")?;
    }

    tracing::debug!(
        faces = texture.faces(),
        face_size,
        width,
        height,
        "Created cubemap sky texture"
    );

    Ok(texture)
}

fn fill_cubemap_face(image: &RgbaImage, face: u32, size: u32, output: &mut [u8]) {
    let width = image.width() as f32;
    let height = image.height() as f32;

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let (dir_x, dir_y, dir_z) = cubemap_direction(face, x, y, size);
            let color = sample_equirectangular(image, dir_x, dir_y, dir_z, width, height);
            output[idx..idx + 4].copy_from_slice(&color);
        }
    }
}

fn cubemap_direction(face: u32, x: u32, y: u32, size: u32) -> (f32, f32, f32) {
    // Use centered pixel sampling to reduce edge artifacts
    let a = 2.0 * (x as f32 + 0.5) / size as f32 - 1.0;
    let b = 2.0 * (y as f32 + 0.5) / size as f32 - 1.0;

    // Corrected OpenGL-style face orientations for seamless cubemap
    let dir = match face {
        0 => Vec3::new(1.0, -b, -a),  // +X (right)
        1 => Vec3::new(-1.0, -b, a),  // -X (left)
        2 => Vec3::new(a, 1.0, b),    // +Y (top)
        3 => Vec3::new(a, -1.0, -b),  // -Y (bottom)
        4 => Vec3::new(a, -b, 1.0),   // +Z (front)
        5 => Vec3::new(-a, -b, -1.0), // -Z (back)
        _ => Vec3::new(0.0, 0.0, 0.0),
    };

    let dir = dir.normalize();
    (dir.x, dir.y, dir.z)
}

fn sample_equirectangular(
    image: &RgbaImage,
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
    width: f32,
    height: f32,
) -> [u8; 4] {
    let u = 0.5 + dir_z.atan2(dir_x) / (2.0 * PI);
    let v = 0.5 - dir_y.asin() / PI;

    // Ensure proper wrapping for both coordinates to prevent seams
    let u = u.rem_euclid(1.0);
    let v = v.clamp(0.0, 1.0);

    // Use bilinear sampling to reduce hard edges
    sample_bilinear(image, u, v, width, height)
}

fn sample_bilinear(
    image: &image::RgbaImage,
    u: f32,
    v: f32,
    width: f32,
    height: f32,
) -> [u8; 4] {
    // Convert to pixel space
    let x = u * (width - 1.0);
    let y = v * (height - 1.0);

    // Get integer and fractional parts
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min((width - 1.0) as u32);
    let y1 = (y0 + 1).min((height - 1.0) as u32);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    // Sample four neighboring pixels
    let p00 = image.get_pixel(x0, y0).0;
    let p10 = image.get_pixel(x1, y0).0;
    let p01 = image.get_pixel(x0, y1).0;
    let p11 = image.get_pixel(x1, y1).0;

    // Bilinear interpolation
    let mut result = [0u8; 4];
    for i in 0..4 {
        let top = p00[i] as f32 * (1.0 - fx) + p10[i] as f32 * fx;
        let bottom = p01[i] as f32 * (1.0 - fx) + p11[i] as f32 * fx;
        result[i] = (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8;
    }

    result
}

pub fn generate_sky_texture(prompt: String) -> Result<String> {
    let full_prompt = format!(
        "a 360-degree seamless equirectangular sky panorama, 8k resolution, no seams skybox texture for a world described as: {}",
        prompt
    );

    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime for generative request")?;

    runtime.block_on(async move {
        let generator = OpenAiImageGenerator::default();
        let request = ImageGenerationRequest::new(full_prompt.clone())
            .with_output_format(ImageOutputFormat::Url);
        let result = generator.generate_image(&request).await?;

        let image_url = result
            .images
            .first()
            .and_then(|image| match &image.data {
                ImageData::Url(url) => Some(url.clone()),
                _ => None,
            })
            .context("image generation returned no URL")?;

        let bytes = reqwest::get(&image_url)
            .await
            .with_context(|| format!("failed to download generated image from {image_url}"))?
            .bytes()
            .await
            .context("failed to read generated image bytes")?;

        let ktx2_texture =
            convert_to_ktx2(&bytes).context("failed to convert downloaded image to KTX2")?;

        let generated_dir = Path::new("assets").join("cubemaps/generated");
        fs::create_dir_all(&generated_dir)
            .context("failed to create generated texture directory")?;
        let file_path = generated_dir.join("sky.ktx2");
        ktx2_texture
            .write_to_file(&file_path)
            .context("failed to save generated KTX2 texture")?;

        tracing::info!("Generated sky texture saved to {:?}", file_path);

        Ok(GENERATED_TEXTURE_PATH.to_string())
    })
}

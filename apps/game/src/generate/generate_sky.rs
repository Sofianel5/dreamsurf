use std::{fs, path::Path};

use anyhow::{Context, Result};
use generative::{
    ImageData, ImageGenerationRequest, ImageGenerator, ImageOutputFormat, OpenAiImageGenerator,
};
use image::load_from_memory;
use ktx2_rw::{Ktx2Texture, VkFormat};
use tokio::runtime::Builder;

const GENERATED_TEXTURE_PATH: &str = "cubemaps/generated/sky.ktx2";

fn convert_to_ktx2(bytes: &[u8]) -> Result<Ktx2Texture> {
    let image =
        load_from_memory(bytes).context("failed to decode image from downloaded bytes")?;
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();

    let rgba = image.into_raw();

    let mut texture = Ktx2Texture::create(width, height, 1, 1, 6, 1, VkFormat::R8G8B8A8Unorm)
        .context("failed to create Ktx2Texture")?;

    for face in 0..6 {
        texture
            .set_image_data(0, 0, face, &rgba)
            .context("failed to set image data on Ktx2Texture")?;
    }

    tracing::debug!(
        faces = texture.faces(),
        width = width,
        height = height,
        "Created KTX2 sky texture"
    );

    Ok(texture)
}

pub fn generate_sky_texture(prompt: String) -> Result<String> {
    let full_prompt = format!("a sky texture for a world described as: {}", prompt);

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

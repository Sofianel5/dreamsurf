use std::{fs, path::Path};

use anyhow::{Context, Result};
use generative::{
    ImageData, ImageGenerationRequest, ImageGenerator, ImageOutputFormat, OpenAiImageGenerator,
};
use tokio::runtime::Builder;

const GENERATED_TEXTURE_PATH: &str = "textures/generated/ground.png";

pub fn generate_ground_texture(prompt: String) -> Result<String> {

    let full_prompt = format!("a 3mx3m tilable, seamless ground texture for a world described as: {}", prompt);

    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime for generative request")?;

    runtime.block_on(async move {
        let generator = OpenAiImageGenerator::default();
        let request =
            ImageGenerationRequest::new(full_prompt.clone()).with_output_format(ImageOutputFormat::Url);
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

        let generated_dir = Path::new("assets").join("textures/generated");
        fs::create_dir_all(&generated_dir).context("failed to create generated texture directory")?;
        let file_path = generated_dir.join("ground.png");
        fs::write(&file_path, &bytes).context("failed to save generated texture")?;

        tracing::info!("Generated ground texture saved to {:?}", file_path);

        Ok(GENERATED_TEXTURE_PATH.to_string())
    })
}

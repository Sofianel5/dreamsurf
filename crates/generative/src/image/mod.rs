mod openai;

pub use openai::OpenAiImageGenerator;

use async_openai::types::{
    ImageResponseFormat as OpenAiResponseFormat, ImageSize as OpenAiImageSize,
};
use async_trait::async_trait;

use crate::error::GenerativeResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSize {
    Square256,
    Square512,
    Square1024,
    Landscape1792x1024,
    Portrait1024x1792,
}

impl ImageSize {
    pub fn as_openai_size(&self) -> OpenAiImageSize {
        match self {
            ImageSize::Square256 => OpenAiImageSize::S256x256,
            ImageSize::Square512 => OpenAiImageSize::S512x512,
            ImageSize::Square1024 => OpenAiImageSize::S1024x1024,
            ImageSize::Landscape1792x1024 => OpenAiImageSize::S1792x1024,
            ImageSize::Portrait1024x1792 => OpenAiImageSize::S1024x1792,
        }
    }
}

impl Default for ImageSize {
    fn default() -> Self {
        ImageSize::Square1024
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOutputFormat {
    Url,
    Base64Json,
}

impl ImageOutputFormat {
    pub fn as_openai_format(&self) -> OpenAiResponseFormat {
        match self {
            ImageOutputFormat::Url => OpenAiResponseFormat::Url,
            ImageOutputFormat::Base64Json => OpenAiResponseFormat::B64Json,
        }
    }
}

impl Default for ImageOutputFormat {
    fn default() -> Self {
        ImageOutputFormat::Base64Json
    }
}

#[derive(Debug, Clone)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    pub size: ImageSize,
    pub output_format: ImageOutputFormat,
    pub image_count: u8,
}

impl ImageGenerationRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            size: ImageSize::default(),
            output_format: ImageOutputFormat::default(),
            image_count: 1,
        }
    }

    pub fn with_size(mut self, size: ImageSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_output_format(mut self, format: ImageOutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn with_image_count(mut self, count: u8) -> Self {
        self.image_count = count.clamp(1, 10);
        self
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedImage {
    pub data: ImageData,
    pub revised_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImageGenerationResult {
    pub images: Vec<GeneratedImage>,
    pub created_at: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ImageData {
    Url(String),
    Base64(String),
}

#[async_trait]
pub trait ImageGenerator: Send + Sync {
    async fn generate_image(
        &self,
        request: &ImageGenerationRequest,
    ) -> GenerativeResult<ImageGenerationResult>;
}

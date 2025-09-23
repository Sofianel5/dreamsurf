use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    CreateImageRequest, CreateImageRequestArgs, Image, ImageModel, ImagesResponse,
};

use crate::error::{GenerativeError, GenerativeResult};

use super::{
    GeneratedImage, ImageData, ImageGenerationRequest, ImageGenerationResult, ImageGenerator,
};

const DEFAULT_IMAGE_MODEL: &str = "dall-e-2";

#[derive(Clone)]
pub struct OpenAiImageGenerator<C = OpenAIConfig>
where
    C: async_openai::config::Config,
{
    client: Client<C>,
    model: ImageModel,
}

impl<C> OpenAiImageGenerator<C>
where
    C: async_openai::config::Config + Clone + Send + Sync + 'static,
{
    pub fn new(client: Client<C>) -> Self {
        Self {
            client,
            model: ImageModel::Other(DEFAULT_IMAGE_MODEL.to_string()),
        }
    }

    pub fn with_model(client: Client<C>, model: ImageModel) -> Self {
        Self { client, model }
    }

    pub fn with_model_name(client: Client<C>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: ImageModel::Other(model.into()),
        }
    }

    pub fn client(&self) -> &Client<C> {
        &self.client
    }

    pub fn model(&self) -> &ImageModel {
        &self.model
    }
}

impl Default for OpenAiImageGenerator<OpenAIConfig> {
    fn default() -> Self {
        Self::new(Client::new())
    }
}

#[async_trait::async_trait]
impl<C> ImageGenerator for OpenAiImageGenerator<C>
where
    C: async_openai::config::Config + Clone + Send + Sync + 'static,
{
    async fn generate_image(
        &self,
        request: &ImageGenerationRequest,
    ) -> GenerativeResult<ImageGenerationResult> {
        let openai_response = self
            .client
            .images()
            .create(build_request(request, &self.model)?)
            .await?;

        parse_response(openai_response)
    }
}

fn build_request(
    request: &ImageGenerationRequest,
    model: &ImageModel,
) -> GenerativeResult<CreateImageRequest> {
    let mut builder = CreateImageRequestArgs::default();
    builder.prompt(request.prompt.clone());
    builder.model(model.clone());
    if request.image_count > 1 {
        builder.n(request.image_count);
    }
    builder.size(request.size.as_openai_size());
    if supports_response_format(model) {
        builder.response_format(request.output_format.as_openai_format());
    }

    builder.build().map_err(GenerativeError::from)
}

fn parse_response(response: ImagesResponse) -> GenerativeResult<ImageGenerationResult> {
    let images = response
        .data
        .into_iter()
        .map(|image| match image.as_ref() {
            Image::Url {
                url,
                revised_prompt,
            } => Ok(GeneratedImage {
                data: ImageData::Url(url.clone()),
                revised_prompt: revised_prompt.clone(),
            }),
            Image::B64Json {
                b64_json,
                revised_prompt,
            } => Ok(GeneratedImage {
                data: ImageData::Base64(b64_json.as_ref().clone()),
                revised_prompt: revised_prompt.clone(),
            }),
        })
        .collect::<Result<Vec<_>, GenerativeError>>()?;

    Ok(ImageGenerationResult {
        images,
        created_at: Some(u64::from(response.created)),
    })
}

fn supports_response_format(model: &ImageModel) -> bool {
    match model {
        ImageModel::DallE2 | ImageModel::DallE3 => true,
        ImageModel::Other(name) => name != "gpt-image-1",
    }
}

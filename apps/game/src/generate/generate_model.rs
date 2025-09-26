use std::{fs, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde_json::json;
use tokio::{runtime::Builder, time::sleep};
use uuid::Uuid;

const MESHY_BASE_URL: &str = "https://api.meshy.ai/openapi/v2/text-to-3d";
const GENERATED_MODEL_DIR: &str = "models/generated";
const PREVIEW_FILENAME: &str = "preview.glb";
const REFINED_FILENAME: &str = "refined.glb";
const POLL_INTERVAL: Duration = Duration::from_secs(5);

pub struct GeneratedModelPaths {
    pub preview_path: String,
    pub refined_path: String,
}

pub fn generate_prop(prompt: String) -> Result<GeneratedModelPaths> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime for Meshy pipeline")?;

    runtime.block_on(async move {
        let api_key = std::env::var("MESHY_API_KEY")
            .context("environment variable MESHY_API_KEY is not set")?;

        let client = build_client(&api_key)?;

        let preview_task_id = create_preview_task(&client, &prompt).await?;
        let preview_task = poll_task_until_finished(&client, &preview_task_id).await?;
        let preview_url =
            extract_glb_url(&preview_task).context("Meshy preview response missing glb URL")?;
        let preview_bytes = download_model(&client, &preview_url).await?;

        let refined_task_id = create_refine_task(&client, &preview_task_id).await?;
        let refined_task = poll_task_until_finished(&client, &refined_task_id).await?;
        let refined_url =
            extract_glb_url(&refined_task).context("Meshy refine response missing glb URL")?;
        let refined_bytes = download_model(&client, &refined_url).await?;

        let generation_id = Uuid::new_v4().to_string();
        let generated_dir = Path::new("assets")
            .join(GENERATED_MODEL_DIR)
            .join(&generation_id);
        fs::create_dir_all(&generated_dir)
            .with_context(|| format!("failed to create directory {:?}", generated_dir))?;

        let preview_path = generated_dir.join(PREVIEW_FILENAME);
        fs::write(&preview_path, &preview_bytes)
            .with_context(|| format!("failed to write preview model to {:?}", preview_path))?;

        let refined_path = generated_dir.join(REFINED_FILENAME);
        fs::write(&refined_path, &refined_bytes)
            .with_context(|| format!("failed to write refined model to {:?}", refined_path))?;

        tracing::info!(
            preview = ?preview_path,
            refined = ?refined_path,
            "Generated Meshy 3D model written to disk"
        );

        Ok(GeneratedModelPaths {
            preview_path: format!("{GENERATED_MODEL_DIR}/{generation_id}/{PREVIEW_FILENAME}"),
            refined_path: format!("{GENERATED_MODEL_DIR}/{generation_id}/{REFINED_FILENAME}"),
        })
    })
}

fn build_client(api_key: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&format!("Bearer {api_key}"))
        .context("invalid MESHY_API_KEY for Authorization header")?;
    headers.insert(AUTHORIZATION, value);

    Client::builder()
        .default_headers(headers)
        .build()
        .context("failed to build reqwest client for Meshy")
}

async fn create_preview_task(client: &Client, prompt: &str) -> Result<String> {
    let body = json!({
        "mode": "preview",
        "prompt": prompt,
        "negative_prompt": "low quality, low resolution, low poly, ugly",
        "art_style": "realistic",
        "should_remesh": true,
    });

    let response = client
        .post(MESHY_BASE_URL)
        .json(&body)
        .send()
        .await
        .context("failed to send preview creation request to Meshy")?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read error body>".into());
        bail!("Meshy preview creation request failed ({status}): {body}");
    }

    let value: serde_json::Value = response
        .json()
        .await
        .context("failed to deserialize Meshy preview creation response")?;

    extract_task_id(&value)
        .with_context(|| format!("Meshy preview creation response missing task id: {value}"))
}

async fn create_refine_task(client: &Client, preview_task_id: &str) -> Result<String> {
    let body = json!({
        "mode": "refine",
        "preview_task_id": preview_task_id,
    });

    let response = client
        .post(MESHY_BASE_URL)
        .json(&body)
        .send()
        .await
        .context("failed to send refine creation request to Meshy")?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read error body>".into());
        bail!("Meshy refine creation request failed ({status}): {body}");
    }

    let value: serde_json::Value = response
        .json()
        .await
        .context("failed to deserialize Meshy refine creation response")?;

    extract_task_id(&value)
        .with_context(|| format!("Meshy refine creation response missing task id: {value}"))
}

fn extract_task_id(value: &serde_json::Value) -> Option<String> {
    value
        .get("result")
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
}

async fn poll_task_until_finished(client: &Client, task_id: &str) -> Result<serde_json::Value> {
    loop {
        let response = client
            .get(format!("{MESHY_BASE_URL}/{task_id}"))
            .send()
            .await
            .with_context(|| format!("failed to poll Meshy task status for {task_id}"))?;

        let response = response
            .error_for_status()
            .with_context(|| format!("Meshy task status check failed for {task_id}"))?;

        let payload: serde_json::Value = response
            .json()
            .await
            .context("failed to deserialize Meshy task status response")?;

        let status = payload
            .get("status")
            .and_then(|status| status.as_str())
            .unwrap_or_default();

        match status {
            "SUCCEEDED" => return Ok(payload),
            "FAILED" => {
                let reason = payload
                    .get("failure_reason")
                    .or_else(|| payload.get("message"))
                    .and_then(|reason| reason.as_str())
                    .unwrap_or("Meshy task failed for unspecified reason");
                bail!("Meshy task {task_id} failed: {reason}");
            }
            other => {
                let progress = payload
                    .get("progress")
                    .and_then(|progress| progress.as_f64())
                    .unwrap_or(0.0);
                tracing::debug!(
                    task_id,
                    status = other,
                    progress,
                    "Meshy task still running"
                );
                sleep(POLL_INTERVAL).await;
            }
        }
    }
}

async fn download_model(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to download model from {url}"))?;

    let response = response
        .error_for_status()
        .with_context(|| format!("Meshy model download returned error for {url}"))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("failed to read Meshy model bytes from {url}"))?;

    Ok(bytes.to_vec())
}

fn extract_glb_url(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("model_urls")
        .and_then(|urls| urls.get("glb"))
        .and_then(|url| url.as_str())
        .map(|url| url.to_string())
}

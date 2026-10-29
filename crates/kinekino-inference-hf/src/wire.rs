//! HF request bodies and response parsing (router.huggingface.co shapes).
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use kinekino_inference::port::Output;
use kinekino_inference::{Chunk, InferenceError, InferenceRequest};
use serde_json::{Value, json};

/// `(path, body)` for a request; `None` = task not offered by HF Inference Providers.
pub fn encode(req: &InferenceRequest) -> Option<(String, Value)> {
    Some(match req {
        InferenceRequest::Transcribe { model, audio } => (
            format!("/hf-inference/models/{model}"),
            json!({"inputs": STANDARD.encode(audio), "parameters": {"return_timestamps": true}}),
        ),
        InferenceRequest::Generate { model, prompt } => (
            "/v1/chat/completions".into(),
            json!({"model": model, "messages": [{"role": "user", "content": prompt}]}),
        ),
        InferenceRequest::Ocr { model, image } => (
            "/v1/chat/completions".into(),
            json!({"model": model, "messages": [{"role": "user", "content": [
                {"type": "text", "text": "Extract all visible text verbatim."},
                {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", STANDARD.encode(image))}}
            ]}]}),
        ),
        InferenceRequest::Embed { model, text } => (
            format!("/hf-inference/models/{model}/pipeline/feature-extraction"),
            json!({"inputs": text}),
        ),
        InferenceRequest::Diarize { .. } | InferenceRequest::SegmentSubject { .. } => return None,
    })
}

fn proto(m: &str) -> InferenceError {
    InferenceError::Protocol(m.into())
}

pub fn decode(req: &InferenceRequest, body: &[u8]) -> Result<Output, InferenceError> {
    let v: Value = serde_json::from_slice(body).map_err(|e| proto(&e.to_string()))?;
    match req {
        InferenceRequest::Transcribe { .. } => {
            let text = v["text"]
                .as_str()
                .ok_or_else(|| proto("missing text"))?
                .to_string();
            let chunks = v["chunks"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|c| {
                            let ts = &c["timestamp"];
                            Ok(Chunk {
                                start: ts[0].as_f64().ok_or_else(|| proto("chunk start"))?,
                                end: ts[1].as_f64().ok_or_else(|| proto("chunk end"))?,
                                text: c["text"].as_str().unwrap_or_default().trim().to_string(),
                            })
                        })
                        .collect::<Result<Vec<_>, InferenceError>>()
                })
                .transpose()?
                .unwrap_or_default();
            Ok(Output::Transcribe {
                text: text.trim().to_string(),
                chunks,
            })
        }
        InferenceRequest::Generate { .. } | InferenceRequest::Ocr { .. } => {
            let text = v["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| proto("missing choices[0].message.content"))?
                .to_string();
            Ok(match req {
                InferenceRequest::Ocr { .. } => Output::Ocr { text },
                _ => Output::Generate { text },
            })
        }
        InferenceRequest::Embed { .. } => {
            // Either `[f..]` or `[[f..]]` (batched); take the first row.
            let row = match v.as_array().and_then(|a| a.first()) {
                Some(Value::Array(inner)) => inner.clone(),
                _ => v
                    .as_array()
                    .cloned()
                    .ok_or_else(|| proto("embedding array"))?,
            };
            let vector = row
                .iter()
                .map(|x| {
                    x.as_f64()
                        .map(|f| f as f32)
                        .ok_or_else(|| proto("embedding value"))
                })
                .collect::<Result<_, _>>()?;
            Ok(Output::Embed { vector })
        }
        InferenceRequest::Diarize { .. } | InferenceRequest::SegmentSubject { .. } => {
            Err(InferenceError::Unsupported(req.task().name().into()))
        }
    }
}

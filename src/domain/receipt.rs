use tracing::info;

// fn file_path(path: &str) -> PathBuf {
//     let mut abs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//     abs_path.push(path);
//     info!("Using path {:?}", abs_path);
//     abs_path
// }

// pub fn parse_image(path: &str) -> Result<(), AppError> {
//     // Use the `download-models.sh` script to download the models.
//     let detection_model_path = file_path("text-detection.rten");
//     let rec_model_path = file_path("text-recognition.rten");
//
//     let detection_model = Model::load_file(detection_model_path)?;
//     let recognition_model = Model::load_file(rec_model_path)?;
//
//     let engine = OcrEngine::new(OcrEngineParams {
//         detection_model: Some(detection_model),
//         recognition_model: Some(recognition_model),
//         ..Default::default()
//     })?;
//
//     // Read image using image-rs library, and convert to RGB if not already
//     // in that format.
//     let img = image::open(path).map(|image| image.into_rgb8())?;
//
//     // Apply standard image pre-processing expected by this library (convert
//     // to greyscale, map range to [-0.5, 0.5]).
//     let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())?;
//     let ocr_input = engine.prepare_input(img_source)?;
//
//     // Detect and recognize text. If you only need the text and don't need any
//     // layout information, you can also use `engine.get_text(&ocr_input)`,
//     // which returns all the text in an image as a single string.
//
//     // Get oriented bounding boxes of text words in input image.
//     let word_rects = engine.detect_words(&ocr_input)?;
//
//     // Group words into lines. Each line is represented by a list of word
//     // bounding boxes.
//     let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
//
//     // Recognize the characters in each line.
//     let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;
//
//     for line in line_texts
//         .iter()
//         .flatten()
//         // Filter likely spurious detections. With future model improvements
//         // this should become unnecessary.
//         .filter(|l| l.to_string().len() > 1)
//     {
//         info!("{}", line);
//     }
//     Ok(())
// }

use crate::application::errors::AppError;
use async_openai::Client;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::log::error;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub store: String,
    pub location: String,
    pub date_time: String,
    pub positions: Vec<Position>,
    pub total: f64,
    pub other: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub name: String,
    pub amount: i16,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReceiptOcrResponse {
    pub final_answer: String,
    pub receipts: Vec<Receipt>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ImageOcrJsonSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    pub r#type: String,
}

pub async fn call_openai(base64_image: &str) -> Result<Option<ReceiptOcrResponse>, AppError> {
    let schema = schema_for!(ReceiptOcrResponse);
    let schema_value = serde_json::to_value(&schema)?;
    let response_format = ImageOcrJsonSchema {
        description: None,
        name: "receipt_ocr".into(),
        schema: Some(schema_value),
        strict: Some(true),
        r#type: "json_schema".to_string(),
    };

    let client = Client::new();

    let data_url = format!("data:image/jpeg;base64,{}", base64_image);

    let request_body = json!({
        "model": "gpt-4.1",
        "input": [
            {
                "role": "user",
                "content": [
                    { "type": "input_text", "text": "You are a receipt scanner, try to extract all information from the image and create a structured response?" },
                    { "type": "input_image", "image_url": data_url }
                ]
            }
        ],
        "text": {
            "format": response_format
        }
    });

    // info!("raw request_body:\n{}", serde_json::to_string_pretty(&request_body)?);

    let resp_json: serde_json::Value = client.responses().create_byot(request_body).await?;

    // info!("raw response:\n{}", serde_json::to_string_pretty(&resp_json)?);

    if let Some(output_text) = resp_json.get("output_text").and_then(|v| v.as_str()) {
        info!("output_text: {}", output_text);
    } else if let Some(output_array) = resp_json.get("output").and_then(|v| v.as_array()) {
        for item in output_array {
            if let Some(content) = item.get("content").and_then(|v| v.as_array()) {
                for part in content {
                    if let Some(part_text) = part.get("text").and_then(|t| t.as_str()) {
                        let obj: ReceiptOcrResponse = serde_json::from_str(part_text).unwrap();
                        info!("part text: {:?}", obj);
                        return Ok(Some(obj));
                    }
                }
            }
            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                info!("item text: {}", text);
            }
        }
    } else {
        error!("Could not find textual output in response; inspect raw JSON above.");
    }

    Ok(None)
}

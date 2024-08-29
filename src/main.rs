use std::{borrow::Cow, path::PathBuf};

use anyhow;
use arboard::Clipboard;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use regex::Regex;

use rten::Model;
use slint::SharedString;

slint::include_modules!();

const DETECTION_MODEL_DATA: &[u8] = include_bytes!("../examples/text-detection.rten");
const RECOGNITION_MODEL_DATA: &[u8] = include_bytes!("../examples/text-recognition.rten");

fn get_screenshot_from_clipboard() -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;

    let image_data = clipboard.get_image()?;

    let width = image_data.width.try_into()?;
    let height = image_data.height.try_into()?;

    let buffer_data: Vec<u8> = match image_data.bytes {
        Cow::Borrowed(bytes) => bytes.to_vec(), // Se for uma referência, converta para Vec
        Cow::Owned(bytes) => bytes,             // Se já for um Vec, mantenha
    };

    let buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, buffer_data)
        .ok_or("Falha ao criar ImageBuffer")?;

    let img = DynamicImage::ImageRgba8(buffer);
    Ok(img)
}

fn get_ocr(app: &App) -> anyhow::Result<()> {
    // Use the `download-models.sh` script to download the models.
    // let detection_model_path = file_path("examples/text-detection.rten");
    // let rec_model_path = file_path("examples/text-recognition.rten");

    // let detection_model = Model::load_file(detection_model_path)?;
    // let recognition_model = Model::load_file(rec_model_path)?;

    let detection_model = Model::load(DETECTION_MODEL_DATA.to_vec())?;
    let recognition_model = Model::load(RECOGNITION_MODEL_DATA.to_vec())?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap();

    let img = get_screenshot_from_clipboard().unwrap();

    let img_source = ImageSource::from_bytes(img.as_bytes(), img.dimensions())?;

    let ocr_input = engine.prepare_input(img_source)?;

    // Detect and recognize text. If you only need the text and don't need any
    // layout information, you can also use `engine.get_text(&ocr_input)`,
    // which returns all the text in an image as a single string.

    // Get oriented bounding boxes of text words in input image.
    let word_rects = engine.detect_words(&ocr_input)?;

    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    for line in line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() > 1)
    {
        let re = Regex::new(r"(\d{4}|\.\d{4})").unwrap();
        if re.is_match(line.to_string().as_str()) {
            let tmp = SharedString::from(line.to_string());
            app.global::<AppField>().set_field(tmp);
            println!("{}", line);
        }
    }
    Ok(())
}

fn ui_xml(app: &App) -> anyhow::Result<()> {
    let myapp = app.clone_strong();
    app.global::<AppField>().on_ocr(move || {
        let localapp = myapp.clone_strong();

        if let Err(_) = get_ocr(&localapp) {}
    });
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let myapp = App::new().unwrap();
    ui_xml(&myapp).ok();

    myapp.run().unwrap();
    Ok(())
}

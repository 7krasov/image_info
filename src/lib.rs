use infer::Infer;
// use serde_json::{Map, Number, Value};
use serde::Serialize;
use std::fs;

pub const CODE_OK: i32 = 0;
pub const CODE_BAD_ARGS: i32 = 1;
pub const CODE_MIME_TYPE_ERROR: i32 = 2;
pub const CODE_DIMENSIONS_ERROR: i32 = 3;

#[derive(Serialize, Default, Debug)]
pub struct InfoResult {
    mime_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    error_message: Option<String>,
    error_code: i32,
}

impl InfoResult {
    fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = Some(width);
        self.height = Some(height);
    }

    pub fn get_width(&self) -> Option<u32> {
        self.width.clone()
    }

    pub fn get_height(&self) -> Option<u32> {
        self.height.clone()
    }

    fn set_mime_type(&mut self, mime_type: String) {
        self.mime_type = Some(mime_type);
    }

    pub fn get_mime_type(&self) -> Option<String> {
        self.mime_type.clone()
    }

    fn set_error_message(&mut self, error_message: String) {
        self.error_message = Some(error_message.to_string());
    }

    pub fn get_error_message(&self) -> Option<String> {
        self.error_message.clone()
    }

    fn set_error_code(&mut self, error_code: i32) {
        self.error_code = error_code;
    }

    pub fn get_error_code(&self) -> i32 {
        self.error_code
    }

    pub fn render(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn process_path(path: &str) -> InfoResult {
    let mut info = InfoResult::default();

    //check if the path exists
    let meta = fs::metadata(path);
    if meta.is_err() {
        info.set_error_message(format!("Error: path does not exist: {}", path));
        info.set_error_code(CODE_BAD_ARGS);
        return info;
    }

    if !meta.unwrap().is_file() {
        info.set_error_message(format!("Error: path is not a file: {}", path));
        info.set_error_code(CODE_BAD_ARGS);
        return info;
    }

    //extract MIME type
    let mime_type = mime_type(path);
    if mime_type.is_err() {
        let msg = mime_type.err().unwrap();
        info.set_error_message(format!("Error getting mime type: {}", msg));
        info.set_error_code(CODE_MIME_TYPE_ERROR);
        return info;
    }

    let mime_type = mime_type.unwrap();
    info.set_mime_type(mime_type.clone());

    //check if the mime type is image/*
    if !mime_type.starts_with("image/") {
        info.set_error_message(format!("Error: mime type is not image/*: {}", mime_type));
        info.set_error_code(CODE_MIME_TYPE_ERROR);
        return info;
    }

    let size = imagesize::size(path);
    if size.is_err() {
        info.set_error_message(format!(
            "Error getting dimensions for path: {:?} {:?}",
            &path,
            size.unwrap_err()
        ));
        info.set_error_code(CODE_DIMENSIONS_ERROR);
        return info;
    }
    let size = size.unwrap();
    // let mut map = Map::new();
    // map.insert("width".to_string(), Value::Number(Number::from(size.width)));
    // map.insert("height".to_string(), Value::Number(Number::from(size.height)));
    // println!("{:?}", map);
    info.set_dimensions(size.width as u32, size.height as u32);

    info
}

fn mime_type(file_path: &str) -> Result<String, String> {
    // Open the file and read its bytes
    let file_bytes = fs::read(file_path);
    if file_bytes.is_err() {
        return Err("Error reading file ".to_string());
    }
    let file_bytes = file_bytes.unwrap();

    // Use `infer` to detect MIME type from file content
    let mime_type = Infer::new().get(&file_bytes);

    if let Some(mime) = mime_type {
        return Ok(mime.mime_type().to_string());
    }

    // Fallback to `tree_magic_mini` if `infer` fails
    let mime = tree_magic_mini::from_u8(&file_bytes);
    if mime.is_empty() {
        return Err("Error getting mime type".to_string());
    }
    Ok(mime.to_string())
}

#[test]
fn should_extract_jpeg_from_jpeg() {
    let path = "tests/fixtures/test1.jpg";
    let mime_type = mime_type(path);
    assert!(mime_type.is_ok());
    assert_eq!(mime_type.unwrap(), "image/jpeg");
}

#[test]
fn should_extract_webp_from_webp() {
    let path = "tests/fixtures/test2.webp";
    let mime_type = mime_type(path);
    assert!(mime_type.is_ok());
    assert_eq!(mime_type.unwrap(), "image/webp");
}

#[test]
fn should_extract_html_from_fake_jpeg() {
    let path = "tests/fixtures/test1_html.jpg";
    let mime_type = mime_type(path);
    assert!(mime_type.is_ok());
    assert_eq!(mime_type.unwrap(), "text/html");
}

#[test]
fn should_extract_jpeg_from_fake_webp() {
    let path = "tests/fixtures/test1_jpg.webp";
    let mime_type = mime_type(path);
    assert!(mime_type.is_ok());
    assert_eq!(mime_type.unwrap(), "image/jpeg");
}

#[test]
fn should_return_jpeg_ok() {
    let info = process_path("tests/fixtures/test1.jpg");
    assert_eq!(info.mime_type.unwrap(), "image/jpeg");
    assert_eq!(info.width.unwrap(), 7724);
    assert_eq!(info.height.unwrap(), 5148);
    assert_eq!(info.error_code, CODE_OK);
}

#[test]
fn should_return_error_on_fake_jpeg() {
    let info = process_path("tests/fixtures/test1_html.jpg");
    assert_eq!(info.mime_type.unwrap(), "text/html");
    assert_eq!(info.width, None);
    assert_eq!(info.height, None);
    assert_eq!(info.error_code, CODE_MIME_TYPE_ERROR);
}

#[test]
fn should_render_jpeg_ok() {
    let info = process_path("tests/fixtures/test1.jpg");
    let render = info.render();
    let expected = r#"{"mime_type":"image/jpeg","width":7724,"height":5148,"error_message":null,"error_code":0}"#;
    assert_eq!(render, expected);
}

#[test]
fn should_render_error_on_fake_jpeg() {
    let info = process_path("tests/fixtures/test1_html.jpg");
    let render = info.render();
    let expected = r#"{"mime_type":"text/html","width":null,"height":null,"error_message":"Error: mime type is not image/*: text/html","error_code":2}"#;
    assert_eq!(render, expected);
}

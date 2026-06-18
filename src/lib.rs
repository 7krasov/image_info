use infer::Infer;
// use serde_json::{Map, Number, Value};
use serde::Serialize;
use std::fs;
#[cfg(feature = "phash")]
use image_hasher::HasherConfig;

pub const CODE_OK: i32 = 0;
pub const CODE_BAD_ARGS: i32 = 1;
pub const CODE_MIME_TYPE_ERROR: i32 = 2;
pub const CODE_DIMENSIONS_ERROR: i32 = 3;
pub const CODE_PHASH_GENERATION_ERROR: i32 = 4;

#[derive(Serialize, Default, Debug)]
pub struct InfoResult {
    mime_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    phash: Option<String>,
    error_message: Option<String>,
    error_code: i32,
}

impl InfoResult {
    fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = Some(width);
        self.height = Some(height);
    }

    pub fn get_width(&self) -> Option<u32> {
        self.width
    }

    pub fn get_height(&self) -> Option<u32> {
        self.height
    }

    pub fn set_phash(&mut self, phash: String) {
        self.phash = Some(phash);
    }

    pub fn get_phash(&self) -> Option<String> {
        self.phash.clone()
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

    #[cfg(feature = "render")]
    pub fn render(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn process_path(path: &str, generate_phash: bool) -> InfoResult {
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

    if generate_phash {
        #[cfg(feature = "phash")]
        match mime_type.as_str() {
            // HEIF and AVIF are both decoded via libvips.
            "image/heif" | "image/avif" => {
                #[cfg(feature = "phash-vips")]
                match calculate_phash_vips(path) {
                    Ok(phash) => info.set_phash(phash),
                    Err(e) => {
                        info.set_error_message(e);
                        info.set_error_code(CODE_PHASH_GENERATION_ERROR);
                    }
                }
                #[cfg(not(feature = "phash-vips"))]
                {
                    info.set_error_message(format!(
                        "libvips backend (phash-vips feature) is not enabled for format {}",
                        mime_type
                    ));
                    info.set_error_code(CODE_PHASH_GENERATION_ERROR);
                }
            }
            _ => match calculate_phash(path) {
                Ok(phash) => info.set_phash(phash),
                Err(e) => {
                    info.set_error_message(e.to_string());
                    info.set_error_code(CODE_PHASH_GENERATION_ERROR);
                }
            },
        }
    }

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

#[cfg(feature = "phash")]
fn calculate_phash(path: &str) -> Result<String, image::ImageError> {
    // let image = image::open(path)?;
    // "image::open" pays attention on file extension.
    // Any fake extension will break the image processing
    let bytes = fs::read(path)?;
    let image = image::load_from_memory(&bytes)?;
    let hasher = HasherConfig::new().to_hasher();
    let hash = hasher.hash_image(&image);
    Ok(hash.to_base64())
}

// libvips must be initialized once per process before any operation. Dropping
// the `VipsApp` calls `vips_shutdown`, which would break later calls, so we keep
// it alive for the whole process lifetime via `mem::forget`.
#[cfg(feature = "phash-vips")]
fn ensure_vips_initialized() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let app = libvips::VipsApp::new("image_info", false)
            .expect("failed to initialize libvips (VipsApp)");
        std::mem::forget(app);
    });
}

#[cfg(feature = "phash-vips")]
pub fn calculate_phash_vips(path: &str) -> Result<String, String> {
    ensure_vips_initialized();

    let image = libvips::VipsImage::new_from_file(path)
        .map_err(|e| format!("Vips load error: {:?}", e))?;

    let jpeg_bytes = libvips::ops::jpegsave_buffer_with_opts(
        &image,
        &libvips::ops::JpegsaveBufferOptions {
            q: 85,
            ..Default::default()
        },
    )
    .map_err(|e| format!("Vips export error: {:?}", e))?;

    let dyn_img = image::load_from_memory(&jpeg_bytes)
        .map_err(|e| format!("Image decode error: {:?}", e))?;

    let hasher = HasherConfig::new().to_hasher();
    let hash = hasher.hash_image(&dyn_img);

    Ok(hash.to_base64())
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
    let info = process_path("tests/fixtures/test1.jpg", false);
    assert_eq!(info.mime_type.unwrap(), "image/jpeg");
    assert_eq!(info.width.unwrap(), 1920);
    assert_eq!(info.height.unwrap(), 1280);
    assert_eq!(info.error_code, CODE_OK);
}

#[test]
fn should_return_error_on_fake_jpeg() {
    let info = process_path("tests/fixtures/test1_html.jpg", false);
    assert_eq!(info.mime_type.unwrap(), "text/html");
    assert_eq!(info.width, None);
    assert_eq!(info.height, None);
    assert_eq!(info.error_code, CODE_MIME_TYPE_ERROR);
}

#[test]
#[cfg(feature = "render")]
fn should_render_jpeg_ok() {
    let info = process_path("tests/fixtures/test1.jpg", false);
    let render = info.render();
    let expected = r#"{"mime_type":"image/jpeg","width":1920,"height":1280,"error_message":null,"error_code":0}"#;
    assert_eq!(render, expected);
}

#[test]
#[cfg(feature = "render")]
fn should_render_error_on_fake_jpeg() {
    let info = process_path("tests/fixtures/test1_html.jpg", false);
    let render = info.render();
    let expected = r#"{"mime_type":"text/html","width":null,"height":null,"error_message":"Error: mime type is not image/*: text/html","error_code":2}"#;
    assert_eq!(render, expected);
}

#[test]
#[cfg(feature = "render")]
#[cfg(feature = "phash")]
fn should_render_phash() {
    let info = process_path("tests/fixtures/test1.jpg", true);
    assert_eq!(info.mime_type.unwrap(), "image/jpeg");
    assert_eq!(info.width.unwrap(), 1920);
    assert_eq!(info.height.unwrap(), 1280);
    assert!(info.phash.is_some());
    assert_eq!(info.phash.unwrap(), "8Ph4HY9Tefw");
    assert_eq!(info.error_code, CODE_OK);
}

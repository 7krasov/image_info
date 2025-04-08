# image_info

A library to extract base information of the image file:
- file size;
- MIME type;
- dimensions.

To get MIME type uses `infer` crate. If the file does not have a valid image MIME type, the `tree_magic_mini` crate is used
by fallback to get the real MIME type as `infer` returns an error if the MIME is not for image (e.g. "plain/text").

To get dimensions uses `imagesize` crate.


## Additional features
- Reports error codes and error messages on failure;
- Can render result as JSON string (useful for using from other languages as wrapper with binary code calling if the FFI is not an option).

## Usage

```rust
fn main() {
  let args: Vec<String> = std::env::args().collect();

  let path: &str = match args.get(1) {
    Some(arg) => arg,
    None => {
      println!("No path argument specified");
      std::process::exit(image_info::CODE_BAD_ARGS);
    }
  };

  let info = image_info::process_path(path);
  println!("{}", info.render());
  //{"mime_type":"image/jpeg","width":7724,"height":5148,"error_message":null,"error_code":0}
  //{"mime_type":"text/html","width":null,"height":null,"error_message":"Error: mime type is not image/*: text/html","error_code":2}
  std::process::exit(info.get_error_code());
}


```


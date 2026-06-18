// Dumps the Debug and JSON (render) output of `process_path` for sample
// fixtures. Useful to regenerate the example output shown in README.md.
//
// Run with:
//   cargo run --example dump --features phash,render

fn main() {
    for path in ["tests/fixtures/test1.jpg", "tests/fixtures/test1_html.jpg"] {
        // second parameter "true" stands for "generate phash"
        let info = image_info::process_path(path, true);
        println!("// {}", path);
        println!("{:?}", info);
        println!("{}", info.render());
        println!();
    }
}
use code_gen::{generate_wasm_function_page, WasmFunctionConfig};
use std::fs;

fn main() {
    // Example JSON for the process_bind function
    let example_json = r#"{
    "string_field": "example value",
    "string_default": "default.txt",
    "counter_field": 2,
    "bool_field": true,
    "int_field": 42,
    "enum_field": "OptionA",
    "vec_field": ["item1", "item2"],
    "uint_field": 10,
    "optional_field": "optional value",
    "flag_field": false,
    "subcommand": null
}"#;

    let config = WasmFunctionConfig {
        function_name: "process_bind".to_string(),
        package_name: "example".to_string(),
        page_title: "WASM Process Function".to_string(),
        example_json: Some(example_json.to_string()),
    };

    let html = generate_wasm_function_page(&config);

    fs::write("generated_ui.html", html).expect("Failed to write HTML file");

    println!("✓ Generated UI written to 'generated_ui.html'");
    println!("\nTo use:");
    println!("  1. Build WASM: wasm-pack build --target web");
    println!("  2. Open generated_ui.html in a browser");
}

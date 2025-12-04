use code_gen::{generate_wasm_function_page, WasmFunctionConfig, extract_field_descriptors_from_command};
use clap::CommandFactory;
use example::Opt;
use std::fs;

fn main() {
    // Extract field descriptors from the Opt struct
    let cmd = Opt::command();
    let fields = extract_field_descriptors_from_command(&cmd);

    let config = WasmFunctionConfig {
        function_name: "process_bind".to_string(),
        package_name: "example".to_string(),
        page_title: "WASM Process Function".to_string(),
        fields,
    };

    let html = generate_wasm_function_page(&config);

    fs::write("generated_ui.html", html).expect("Failed to write HTML file");

    println!("✓ Generated UI written to 'generated_ui.html'");
    println!("\nTo use:");
    println!("  1. Build WASM: wasm-pack build --target web");
    println!("  2. Open generated_ui.html in a browser");
}

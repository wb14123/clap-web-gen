
This project can generate a web UI based on a [Clap](https://docs.rs/clap/latest/clap/) command.

It maps the cli args to HTML input elements. When click a button, it passes in the inputs as a Clap structure and call user defined function (the function need to be compiled to WASM).

## Usage

### 1. Use the macros in your code

```rust
use code_gen::{web_ui_bind, wprintln};

#[web_ui_bind]
pub fn process(input: &YourInputType) {
    wprintln!("Your output here");
    // Use wprintln! instead of println! to capture output in WASM
}
```

The `#[web_ui_bind]` attribute will:
- Keep your function unchanged for CLI use
- Generate a `process_bind` function for WASM
- Capture all `wprintln!` output and return it to the browser

### 2. Create a binary to generate the UI

```rust
use code_gen::{generate_wasm_function_page, WasmFunctionConfig};
use std::fs;

fn main() {
    let config = WasmFunctionConfig {
        function_name: "process_bind".to_string(),
        package_name: "your_package_name".to_string(),
        page_title: "My WASM Function".to_string(),
        example_json: Some(r#"{"field": "value"}"#.to_string()),
    };

    let html = generate_wasm_function_page(&config);
    fs::write("generated_ui.html", html).unwrap();
}
```

### 3. Build and run

```bash
# Generate the HTML UI
cargo run --bin generate_ui

# Build WASM
wasm-pack build --target web

# Open generated_ui.html in browser
```

## Example

There is an example under `example/` project.

Build WASM files with this command first:

```
~/.cargo/bin/wasm-pack build --target web
```

It will create files under `pkg`.

Then run

```
cargo run --bin generate_ui
```

It will generate a `generated_ui.html` file that you can open in the browser that can run the exported `process_bind` with web UI.

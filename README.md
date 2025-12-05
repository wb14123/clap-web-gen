# Clap Web UI Generator

Automatically generate web UIs for your Rust CLI applications using Clap.

This project maps CLI args to HTML input elements. When you click a button, it passes the inputs as a Clap structure and calls your WASM-compiled function.

## Features

- **Single command** to generate web UIs - no manual code needed!
- Supports all Clap field types (string, bool, int, enum, vec, counter, etc.)
- Supports subcommands
- Compiles to WebAssembly for client-side execution
- Auto-discovery of `#[web_ui_bind]` functions

## Quick Start

### 1. Add the macro to your function

```rust
use clap::Parser;
use code_gen::{web_ui_bind, wprintln};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    count: u32,
}

#[web_ui_bind]
pub fn process(args: &Args) {
    wprintln!("Hello, {}!", args.name);
    wprintln!("Count: {}", args.count);
}
```

The `#[web_ui_bind]` attribute will:
- Keep your function unchanged for CLI use
- Generate a `process_bind` function for WASM
- Generate a `generate_process_ui()` function for creating the HTML
- Capture all `wprintln!` output and return it to the browser

You can optionally specify a custom HTML filename:

```rust
#[web_ui_bind(html_name = "custom.html")]
pub fn process(args: &Args) {
    // ...
}
```

If not specified, defaults to `index.html`.

### 2. Generate the web UI with a single command

From your project directory:

```bash
cargo run --package code_gen --bin generate-web-ui
```

This will:
- Scan your source files for `#[web_ui_bind]` functions
- Generate HTML files in the `pkg/` directory (defaults to `index.html`)
- All temporary files go into `target/clap-web-gen/` (gitignored)

### 3. Build WASM and test

```bash
wasm-pack build --target web
# Open the generated HTML files in pkg/ (e.g., pkg/index.html) in your browser
```

## How It Works

1. **Macro expansion**: The `#[web_ui_bind]` macro automatically generates:
   - A WASM binding function (e.g., `process_bind`)
   - A UI generation function (e.g., `generate_process_ui`)

2. **Auto-discovery**: The CLI tool uses `syn` to parse your source files and find all `#[web_ui_bind]` annotations

3. **HTML generation**: Creates a temporary project in `target/clap-web-gen/` that calls your UI generation functions and outputs HTML files

4. **Clean output**: HTML files are generated in the `pkg/` directory alongside your WASM files

## Example

See the `example/` directory for a complete working example with subcommands, various field types, and more.

To try it:

```bash
cd example
cargo run --package code_gen --bin generate-web-ui
wasm-pack build --target web
# Open pkg/index.html in your browser
```

## wprintln! Macro

Use `wprintln!` instead of `println!` in your functions to capture output in both native and WASM builds:

```rust
wprintln!("This works in native and WASM!");
```

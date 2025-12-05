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

### 1. Define your Clap structure and add the macro

```rust
use clap::Parser;
use clap_web_code_gen::{web_ui_bind, wprintln};

#[derive(Parser)]
#[command(about = "A simple greeting CLI", long_about = "This is a more detailed description of what this CLI does")]
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

**Important**: Use the `#[command(about = "...", long_about = "...")]` attributes on your Clap struct:
- `about`: Short description used as the **page title** in the generated web UI
- `long_about`: Detailed description displayed as the **page description** in the web UI

The `#[web_ui_bind]` attribute will:
- Keep your function unchanged for CLI use
- Generate a `process_bind` function for WASM
- Capture all `wprintln!` output and return it to the browser

You can optionally specify a custom HTML filename:

```rust
#[web_ui_bind(html_name = "custom.html")]
pub fn process(args: &Args) {
    // ...
}
```

If not specified, defaults to `index.html`.

### 2. Replace print macros with web-compatible versions

Replace all `print!` and `println!` macros in your function with `wprint!` and `wprintln!`:

```rust
// Before
println!("Hello, {}!", name);

// After
wprintln!("Hello, {}!", name);
```

**Why?** Standard `print!` and `println!` don't work in WebAssembly. The `wprint!` and `wprintln!` macros:
- Capture output in WASM and display it in the browser
- Still work normally in native CLI builds
- Are automatically handled by the `#[web_ui_bind]` macro to return output to the web UI

### 3. Build WASM and test

```bash
wasm-pack build --target web
# Open the generated HTML files in pkg/ (e.g., pkg/index.html) in your browser
```

### 4. Generate the web UI with a single command

From your project directory:

```bash
cargo install clap_web_code_gen
clap-web-gen
```

This will:
- Scan your source files for `#[web_ui_bind]` functions
- Generate HTML files in the `pkg/` directory (defaults to `index.html`)
- All temporary files go into `target/clap-web-gen/` (gitignored)

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
cargo run --package clap_web_code_gen --bin clap-web-gen
wasm-pack build --target web
# Open pkg/index.html in your browser
```

## wprintln! Macro

Use `wprintln!` instead of `println!` in your functions to capture output in both native and WASM builds:

```rust
wprintln!("This works in native and WASM!");
```

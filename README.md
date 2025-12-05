# Clap Web UI Generator

Automatically generate web UIs for Rust CLI applications using [Clap](https://docs.rs/clap/latest/clap/).

This project maps CLI args to HTML input elements. When you click a button, it passes the inputs as a Clap structure and calls your WASM-compiled function.

## Features

- Generate web UI through a single macro `#[web_ui_bind]`.
- Supports all Clap field types (string, bool, int, enum, vec, counter, etc.).
- Supports subcommands.

## Limitations

- The function must be able to compile to WASM and run in a browser.
- Need to replace stdout with macros in this project `wprint!` and `wprintln!`.

## How Clap Structures Map to Web UI

The generator automatically maps your Clap CLI definition to HTML form elements:

**Command-level attributes:**
- `#[command(about = "...")]` → Page title
- `#[command(long_about = "...")]` → Page description/subtitle

**Argument mapping:**
- **Input labels**: Generated from the doc comment (`///`) or `help` attribute if available, otherwise uses the field name
  ```rust
  /// Your name
  #[arg(short, long)]
  name: String,  // Label: "Your name"

  #[arg(short, long)]
  name: String,  // Label: "name"
  ```

- **Positional arguments** (no `short` or `long`) → `<textarea>` element (for multi-line input)
  ```rust
  /// Input text to process
  #[arg(required = true)]
  text: String,  // Renders as textarea
  ```

- **Named arguments** (with `short` or `long`) → `<input>` element
  ```rust
  /// User's email address
  #[arg(short, long)]
  email: String,  // Renders as <input type="text">
  ```

**Field types:**
- `String`, `&str` → Text input or textarea
- `bool` → Checkbox
- Integer types (`u32`, `i32`, etc.) → Number input
- `Vec<T>` → Multiple inputs or comma-separated values
- Enums → Dropdown/select menu
- Counter types → Number input

**Subcommands:**
- Each subcommand becomes a separate section or tab in the UI
- Subcommand fields follow the same mapping rules

## Quick Start

### 1. Prepare your Clap CLI

**If you already have a Clap-based CLI:**

Most CLIs have all their logic in `main.rs`. To use this library, you'll need to:
1. Create a `lib.rs` file if you don't have one
2. Move your Clap structure to `lib.rs`
3. Extract your main logic into a separate function that takes your Clap structure as its **only parameter**
4. Add the `#[web_ui_bind]` macro to this function

**Example `lib.rs`:**

```rust
use clap::Parser;
use clap_web_code_gen::{web_ui_bind, wprintln};

#[derive(Parser)]
#[command(about = "A simple greeting CLI", long_about = "This is a more detailed description of what this CLI does")]
pub struct Args {
    /// Your name
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long)]
    count: u32,
}

// This function MUST:
// - Be defined in lib.rs (not main.rs)
// - Take only your Clap structure as a parameter (e.g., &Args)
// - Be marked as pub
#[web_ui_bind]
pub fn process(args: &Args) {
    wprintln!("Hello, {}!", args.name);
    wprintln!("Count: {}", args.count);
}
```

**Then update your `main.rs` to call this function:**

```rust
use clap::Parser;
use your_crate_name::{Args, process};

fn main() {
    let args = Args::parse();
    process(&args);
}
```

**If you're starting a new CLI from scratch:**

Follow the same structure above - define your Clap structure and processing function in `lib.rs`, and keep `main.rs` simple with just parsing and calling your function.

The `#[web_ui_bind]` macro will:
- Keep your function unchanged for CLI use
- Generate a `process_bind` function for WASM
- Capture all `wprintln!` output and return it to the browser

**Requirements for the function:**
- Must be in `lib.rs` (not `main.rs`)
- Must be `pub`
- Must take exactly **one parameter**: a reference to your Clap structure (e.g., `&Args`, `&YourStruct`)
- Cannot have additional parameters or return values

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

## Release

Check before release:

```
cargo release
```

Then actually upload:

```
cargo release --execute
```

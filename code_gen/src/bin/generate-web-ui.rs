/// Code generator CLI tool for discovering and generating web UIs
///
/// This tool scans Rust source files for #[web_ui_bind] annotations and
/// automatically generates HTML files for web UIs.
///
/// Usage:
///   From your project directory (where you use #[web_ui_bind]):
///     cargo run --package code_gen --bin generate-web-ui
///
/// Or install globally:
///     cargo install --path code_gen
///     cd your_project && generate-web-ui

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::{File, Item, ItemFn};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let only_codegen = args.iter().any(|a| a == "--only-codegen");

    println!("🔍 Web UI Generator");
    println!("   Scanning for #[web_ui_bind] functions...\n");

    // Get current directory (should be run from project root)
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Find the package name from Cargo.toml
    let package_name = get_package_name(&current_dir);
    println!("📦 Package: {}", package_name);

    // Find all Rust source files
    let src_dir = current_dir.join("src");
    if !src_dir.exists() {
        eprintln!("❌ Error: No src/ directory found");
        eprintln!("   Please run this from your project root");
        std::process::exit(1);
    }

    let src_files = find_rust_files(&src_dir);
    println!("📁 Scanning {} file(s)...", src_files.len());

    // Parse files to find web_ui_bind functions
    let bound_functions = find_web_ui_bind_functions(&src_files);

    if bound_functions.is_empty() {
        println!("\n⚠️  No #[web_ui_bind] functions found");
        println!("   Add #[web_ui_bind] to your functions to generate web UIs\n");
        std::process::exit(0);
    }

    println!("\n✓ Found {} function(s) with #[web_ui_bind]:", bound_functions.len());
    for func in &bound_functions {
        println!("  • {}", func.name);
    }

    // Generate the UI generator source file in target directory (gitignored)
    let generator_code = generate_ui_generator_code(&package_name, &bound_functions);

    // Write to target/clap-web-gen/ directory (not src/, to avoid noise)
    let gen_dir = current_dir.join("target/clap-web-gen");
    fs::create_dir_all(&gen_dir).expect("Failed to create target/clap-web-gen directory");

    let generator_path = gen_dir.join("ui_generator.rs");
    fs::write(&generator_path, generator_code)
        .expect("Failed to write generator file");

    if only_codegen {
        println!("\n✅ Code generation complete!");
        println!("   Temporary file: target/clap-web-gen/ui_generator.rs");
        return;
    }

    // Automatically compile and run the generator to create HTML files
    println!("\n🎨 Compiling and running generator...");

    // First, build the project to ensure dependencies are available
    let build_status = Command::new("cargo")
        .arg("build")
        .arg("--lib")
        .current_dir(&current_dir)
        .status();

    if let Err(e) = build_status {
        eprintln!("\n⚠️  Failed to build project: {}", e);
        std::process::exit(1);
    }

    // Compile the temporary generator using cargo-script approach
    let status = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(create_temp_manifest(&gen_dir, &package_name, &current_dir))
        .current_dir(&current_dir)
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("\n✅ Complete! Your web UIs are ready!");
            println!("\nNext steps:");
            println!("  1. Build WASM: wasm-pack build --target web");
            println!("  2. Open the generated *_ui.html files in a browser\n");
        }
        Ok(_) => {
            eprintln!("\n⚠️  HTML generation failed");
        }
        Err(e) => {
            eprintln!("\n⚠️  Failed to run generator: {}", e);
        }
    }
}

#[derive(Debug)]
struct BoundFunction {
    name: String,
}

fn find_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recursively search subdirectories
                files.extend(find_rust_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    files
}

fn find_web_ui_bind_functions(files: &[PathBuf]) -> Vec<BoundFunction> {
    let mut functions = Vec::new();

    for file_path in files {
        if let Ok(content) = fs::read_to_string(file_path) {
            // Parse the file with syn
            if let Ok(ast) = syn::parse_file(&content) {
                functions.extend(extract_web_ui_bind_functions(&ast));
            }
        }
    }

    functions
}

fn extract_web_ui_bind_functions(ast: &File) -> Vec<BoundFunction> {
    let mut functions = Vec::new();

    for item in &ast.items {
        if let Item::Fn(item_fn) = item {
            if has_web_ui_bind_attribute(item_fn) {
                let name = item_fn.sig.ident.to_string();
                functions.push(BoundFunction { name });
            }
        }
    }

    functions
}

fn has_web_ui_bind_attribute(item_fn: &ItemFn) -> bool {
    for attr in &item_fn.attrs {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "web_ui_bind" {
                return true;
            }
        }
    }
    false
}

fn generate_ui_generator_code(package_name: &str, functions: &[BoundFunction]) -> String {
    let mut code = String::new();

    // Add imports
    code.push_str(&format!("use {}::*;\n", package_name));
    code.push_str("use std::fs;\n\n");

    // Add main function
    code.push_str("fn main() {\n");
    code.push_str("    println!(\"🎨 Generating Web UIs...\\n\");\n\n");

    // Generate code for each function
    for func in functions {
        let ui_gen_fn = format!("generate_{}_ui", func.name);
        let output_file = format!("{}_ui.html", func.name);

        code.push_str(&format!("    // Generate UI for {}\n", func.name));
        code.push_str(&format!("    let html = {}(\"{}\", \"{} - Web UI\");\n",
            ui_gen_fn, package_name, func.name));
        code.push_str(&format!("    fs::write(\"{}\", html)\n", output_file));
        code.push_str("        .expect(\"Failed to write HTML file\");\n");
        code.push_str(&format!("    println!(\"  ✓ Generated: {}\");\n\n", output_file));
    }

    code.push_str("    println!(\"\\n✅ All HTML files generated successfully!\");\n");
    code.push_str("    println!(\"\\nNext steps:\");\n");
    code.push_str("    println!(\"  1. Build WASM: wasm-pack build --target web\");\n");
    code.push_str("    println!(\"  2. Open the HTML files in a browser\");\n");
    code.push_str("}\n");

    code
}

fn get_package_name(project_root: &Path) -> String {
    let cargo_toml = project_root.join("Cargo.toml");

    if let Ok(content) = fs::read_to_string(cargo_toml) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name") {
                if let Some(eq_pos) = line.find('=') {
                    let value = line[eq_pos + 1..].trim();
                    // Remove quotes
                    let name = value.trim_matches('"').trim_matches('\'');
                    return name.to_string();
                }
            }
        }
    }

    "unknown".to_string()
}

fn create_temp_manifest(gen_dir: &Path, package_name: &str, project_root: &Path) -> PathBuf {
    let manifest_content = format!(
        r#"[package]
name = "clap-web-gen-temp"
version = "0.1.0"
edition = "2024"

# Empty workspace to mark this as standalone, not part of parent workspace
[workspace]

[[bin]]
name = "ui_generator"
path = "ui_generator.rs"

[dependencies]
{} = {{ path = "{}" }}
"#,
        package_name,
        project_root.display()
    );

    let manifest_path = gen_dir.join("Cargo.toml");
    fs::write(&manifest_path, manifest_content)
        .expect("Failed to write temporary Cargo.toml");

    manifest_path
}

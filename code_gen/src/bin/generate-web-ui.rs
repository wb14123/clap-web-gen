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

    println!("Web UI Generator");
    println!("Scanning for #[web_ui_bind] functions...\n");

    // Get current directory (should be run from project root)
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Find the package name from Cargo.toml
    let package_name = get_package_name(&current_dir);
    println!("Package: {}", package_name);

    // Find all Rust source files
    let src_dir = current_dir.join("src");
    if !src_dir.exists() {
        eprintln!("Error: No src/ directory found");
        eprintln!("Please run this from your project root");
        std::process::exit(1);
    }

    let src_files = find_rust_files(&src_dir);
    println!("Scanning {} file(s)...", src_files.len());

    // Parse files to find web_ui_bind functions
    let bound_functions = find_web_ui_bind_functions(&src_files, &src_dir);

    if bound_functions.is_empty() {
        println!("\nNo #[web_ui_bind] functions found");
        println!("Add #[web_ui_bind] to your functions to generate web UIs\n");
        std::process::exit(0);
    }

    // Check if any functions are in main.rs (binary target)
    let binary_functions: Vec<_> = bound_functions
        .iter()
        .filter(|f| f.module_path == "__BINARY_TARGET__")
        .collect();

    if !binary_functions.is_empty() {
        eprintln!("\nError: #[web_ui_bind] functions found in main.rs");
        eprintln!("Functions in main.rs are part of the binary target and cannot");
        eprintln!("be used by the web UI generator.\n");
        eprintln!("The following functions need to be moved to lib.rs or a library module:");
        for func in &binary_functions {
            eprintln!("  - {}", func.name);
        }
        eprintln!("\nSolution:");
        eprintln!("1. Move your CLI struct and #[web_ui_bind] function to src/lib.rs");
        eprintln!("2. Re-export them in main.rs if needed: pub use {}::{{Cli, run}};", package_name);
        eprintln!("3. Update main.rs to call the function from the library\n");
        std::process::exit(1);
    }

    println!("\nFound {} function(s) with #[web_ui_bind]:", bound_functions.len());
    for func in &bound_functions {
        println!("  - {} -> pkg/{}", func.name, func.html_name);
    }

    // Check for HTML filename conflicts
    let mut html_names = std::collections::HashMap::new();
    for func in &bound_functions {
        if let Some(existing) = html_names.insert(&func.html_name, &func.name) {
            eprintln!("\nError: HTML filename conflict detected!");
            eprintln!("Multiple functions are configured to generate 'pkg/{}':", func.html_name);
            eprintln!("  - Function '{}' ", existing);
            eprintln!("  - Function '{}' ", func.name);
            eprintln!("\nSolution:");
            eprintln!("Specify different HTML filenames using the html_name parameter:");
            eprintln!("  #[web_ui_bind(html_name = \"function1.html\")]");
            eprintln!("  #[web_ui_bind(html_name = \"function2.html\")]\n");
            std::process::exit(1);
        }
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
        println!("\nCode generation complete!");
        println!("Temporary file: target/clap-web-gen/ui_generator.rs");
        return;
    }

    // Automatically compile and run the generator to create HTML files
    println!("\nCompiling and running generator...");

    // First, build the project to ensure dependencies are available
    let build_status = Command::new("cargo")
        .arg("build")
        .arg("--lib")
        .current_dir(&current_dir)
        .status();

    if let Err(e) = build_status {
        eprintln!("\nFailed to build project: {}", e);
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
            println!("\nHTML generation finished.");
        }
        Ok(_) => {
            eprintln!("\nHTML generation failed");
        }
        Err(e) => {
            eprintln!("\nFailed to run generator: {}", e);
        }
    }
}

#[derive(Debug)]
struct BoundFunction {
    name: String,
    module_path: String,  // e.g., "commands::run" or "" for crate root
    html_name: String,    // HTML filename (defaults to "index.html")
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

fn find_web_ui_bind_functions(files: &[PathBuf], src_dir: &Path) -> Vec<BoundFunction> {
    let mut functions = Vec::new();

    for file_path in files {
        if let Ok(content) = fs::read_to_string(file_path) {
            // Parse the file with syn
            if let Ok(ast) = syn::parse_file(&content) {
                let module_path = calculate_module_path(file_path, src_dir);
                functions.extend(extract_web_ui_bind_functions(&ast, &module_path));
            }
        }
    }

    functions
}

fn calculate_module_path(file_path: &Path, src_dir: &Path) -> String {
    // Get the relative path from src/ directory
    let rel_path = file_path.strip_prefix(src_dir).unwrap_or(file_path);

    // Convert path to module path
    let path_str = rel_path.to_str().unwrap_or("");

    // Handle main.rs - this is a special marker that we'll check later
    if path_str == "main.rs" {
        return "__BINARY_TARGET__".to_string();
    }

    // Handle lib.rs (crate root)
    if path_str == "lib.rs" {
        return String::new();
    }

    // Remove .rs extension and convert path separators to ::
    let module_path = path_str
        .trim_end_matches(".rs")
        .replace(std::path::MAIN_SEPARATOR, "::");

    // Handle mod.rs files (e.g., src/commands/mod.rs -> commands)
    if module_path.ends_with("::mod") {
        module_path.trim_end_matches("::mod").to_string()
    } else {
        module_path
    }
}

fn extract_web_ui_bind_functions(ast: &File, module_path: &str) -> Vec<BoundFunction> {
    let mut functions = Vec::new();

    for item in &ast.items {
        if let Item::Fn(item_fn) = item {
            if let Some(html_name) = get_web_ui_bind_html_name(item_fn) {
                let name = item_fn.sig.ident.to_string();
                functions.push(BoundFunction {
                    name,
                    module_path: module_path.to_string(),
                    html_name,
                });
            }
        }
    }

    functions
}

fn get_web_ui_bind_html_name(item_fn: &ItemFn) -> Option<String> {
    for attr in &item_fn.attrs {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "web_ui_bind" {
                // Parse the attribute arguments
                if let Ok(meta_list) = attr.meta.require_list() {
                    // Parse tokens as nested meta items
                    let tokens = &meta_list.tokens;
                    let tokens_str = tokens.to_string();

                    // Simple parsing: look for html_name = "value"
                    if let Some(start) = tokens_str.find("html_name") {
                        let after_name = &tokens_str[start..];
                        if let Some(eq_pos) = after_name.find('=') {
                            let after_eq = after_name[eq_pos + 1..].trim();
                            // Extract quoted string
                            if let Some(value) = extract_quoted_string(after_eq) {
                                return Some(value);
                            }
                        }
                    }
                } else if attr.meta.require_path_only().is_ok() {
                    // No arguments, use default
                    return Some("index.html".to_string());
                }

                // If we found the attribute but couldn't parse args, use default
                return Some("index.html".to_string());
            }
        }
    }
    None
}

fn extract_quoted_string(s: &str) -> Option<String> {
    let s = s.trim();
    if s.starts_with('"') {
        if let Some(end_quote) = s[1..].find('"') {
            return Some(s[1..=end_quote].to_string());
        }
    }
    None
}

fn generate_ui_generator_code(package_name: &str, functions: &[BoundFunction]) -> String {
    let mut code = String::new();

    // Add imports
    code.push_str("use std::fs;\n\n");

    // Add main function
    code.push_str("fn main() {\n");
    code.push_str("    println!(\"Generating Web UIs...\\n\");\n\n");
    code.push_str("    // Create pkg directory if it doesn't exist\n");
    code.push_str("    fs::create_dir_all(\"pkg\")\n");
    code.push_str("        .expect(\"Failed to create pkg directory\");\n\n");

    // Convert package name to valid Rust identifier (hyphens -> underscores)
    let rust_package_name = package_name.replace('-', "_");

    // Generate code for each function
    for func in functions {
        let ui_gen_fn = format!("generate_{}_ui", func.name);
        let output_file = format!("pkg/{}", func.html_name);

        // Build fully qualified function path
        let full_fn_path = if func.module_path.is_empty() {
            format!("{}::{}", rust_package_name, ui_gen_fn)
        } else {
            format!("{}::{}::{}", rust_package_name, func.module_path, ui_gen_fn)
        };

        code.push_str(&format!("    // Generate UI for {}\n", func.name));
        code.push_str(&format!("    let html = {}(\"{}\", \"\");\n",
            full_fn_path, package_name));
        code.push_str(&format!("    fs::write(\"{}\", html)\n", output_file));
        code.push_str("        .expect(\"Failed to write HTML file\");\n");
        code.push_str(&format!("    println!(\"  Generated: {}\");\n\n", output_file));
    }

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
    // Find the code_gen dependency from the user's Cargo.toml
    let code_gen_dep = find_code_gen_dependency(project_root);

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
code_gen = {}
"#,
        package_name,
        project_root.display(),
        code_gen_dep
    );

    let manifest_path = gen_dir.join("Cargo.toml");
    fs::write(&manifest_path, manifest_content)
        .expect("Failed to write temporary Cargo.toml");

    manifest_path
}

fn find_code_gen_dependency(project_root: &Path) -> String {
    let cargo_toml = project_root.join("Cargo.toml");

    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        // Simple parsing to find code_gen dependency
        let mut in_dependencies = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check if we're entering a dependencies section
            if trimmed == "[dependencies]" || trimmed == "[dev-dependencies]" {
                in_dependencies = true;
                continue;
            }

            // Check if we're leaving dependencies section
            if trimmed.starts_with('[') && in_dependencies {
                in_dependencies = false;
                continue;
            }

            // Look for code_gen dependency
            if in_dependencies && trimmed.starts_with("code_gen") {
                if let Some(eq_pos) = trimmed.find('=') {
                    let dep_spec = trimmed[eq_pos + 1..].trim();

                    // If it's a path dependency, resolve to absolute path
                    if dep_spec.contains("path") {
                        return resolve_path_dependency(dep_spec, project_root);
                    }

                    return dep_spec.to_string();
                }
            }
        }
    }

    // Fallback: assume code_gen is in a common location relative to the user's project
    // This might not work in all cases, but provides a reasonable default
    eprintln!("Warning: Could not find code_gen dependency in Cargo.toml");
    eprintln!("Please ensure code_gen is listed in your dependencies");
    r#"{ path = "../clap-web-gen/code_gen" }"#.to_string()
}

fn resolve_path_dependency(dep_spec: &str, project_root: &Path) -> String {
    // Parse the path from the dependency spec
    // Handle formats like: { path = "../clap-web-gen/code_gen" }

    if let Some(path_start) = dep_spec.find("path") {
        let after_path = &dep_spec[path_start..];
        if let Some(eq_pos) = after_path.find('=') {
            let after_eq = &after_path[eq_pos + 1..];

            // Extract the path value (could be quoted or in braces)
            let path_value = after_eq
                .trim()
                .trim_start_matches('{')
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            // Find the end of the path (before comma or closing brace)
            let path_end = path_value
                .find(',')
                .or_else(|| path_value.find('}'))
                .unwrap_or(path_value.len());

            let rel_path = path_value[..path_end].trim().trim_matches('"').trim_matches('\'');

            // Resolve to absolute path
            let abs_path = project_root.join(rel_path);
            let abs_path = abs_path.canonicalize().unwrap_or(abs_path);

            return format!(r#"{{ path = "{}" }}"#, abs_path.display());
        }
    }

    // If we can't parse it, return as-is
    dep_spec.to_string()
}

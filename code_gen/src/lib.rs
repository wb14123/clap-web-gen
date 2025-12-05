// Re-export the procedural macros
pub use code_gen_macro::{web_ui_bind, wprintln};

// Re-export paste for use in macros
#[doc(hidden)]
pub use paste;

use serde::Serialize;
use clap::{Command, Arg, ArgAction};
use maud::{html, Markup, PreEscaped, DOCTYPE};

/// Represents a possible value for an enum field
#[derive(Debug, Clone, Serialize)]
pub struct EnumOption {
    /// The actual value (e.g., "option-a")
    pub value: String,
    /// The help text / description for this option (e.g., "This is Option A")
    pub help: String,
}

/// Type of CLI field for form generation
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "options")]
pub enum FieldType {
    /// String field (text input)
    String,
    /// Boolean field (checkbox)
    Bool,
    /// Integer field (number input)
    Integer,
    /// Counter field (number input, flag repeated N times)
    Counter,
    /// Enum field with possible values
    Enum(Vec<EnumOption>),
    /// Vec field (can add multiple values)
    Vec,
}

/// Descriptor for a CLI field
#[derive(Debug, Clone, Serialize)]
pub struct FieldDescriptor {
    /// Field name (used as HTML id and for CLI args)
    pub name: String,
    /// Short flag (e.g., 's' for -s)
    pub short: Option<char>,
    /// Long flag (e.g., "string-field" for --string-field)
    pub long: Option<String>,
    /// Help text / description
    pub help: String,
    /// Field type
    pub field_type: FieldType,
    /// Default value (as string)
    pub default_value: Option<String>,
    /// Whether the field is required
    pub required: bool,
    /// Whether this is a positional argument (not a flag)
    #[serde(default)]
    pub is_positional: bool,
}

/// Descriptor for a subcommand
#[derive(Debug, Clone, Serialize)]
pub struct SubcommandDescriptor {
    /// Subcommand name (e.g., "sub1", "add", "remove")
    pub name: String,
    /// Help text / description for this subcommand
    pub help: String,
    /// Fields specific to this subcommand
    pub fields: Vec<FieldDescriptor>,
}

/// Configuration for generating a WASM function web interface
pub struct WasmFunctionConfig {
    /// The name of the WASM function to call (e.g., "process")
    pub function_name: String,
    /// The package name (used in import path, e.g., "example" for "./example.js" when HTML is in pkg/)
    pub package_name: String,
    /// The title to display on the web page
    pub page_title: String,
    /// Field descriptors for generating form inputs
    pub fields: Vec<FieldDescriptor>,
    /// Subcommand descriptors (if any)
    pub subcommands: Vec<SubcommandDescriptor>,
}

/// Extracts field descriptors from a Clap Command
///
/// This function introspects a Clap Command at runtime to extract
/// information about all arguments (short, long, help, type, defaults, etc.)
/// and converts them into FieldDescriptor objects suitable for web UI generation.
///
/// # Arguments
///
/// * `command` - A Clap Command object (typically obtained via `CommandFactory::command()`)
///
/// # Returns
///
/// A Vec of FieldDescriptor objects representing all CLI arguments
///
/// # Example
///
/// ```
/// use clap::{Parser, CommandFactory};
/// use code_gen::extract_field_descriptors_from_command;
///
/// #[derive(Parser)]
/// struct MyArgs {
///     #[arg(short, long)]
///     name: String,
/// }
///
/// let cmd = MyArgs::command();
/// let fields = extract_field_descriptors_from_command(&cmd);
/// ```
pub fn extract_field_descriptors_from_command(command: &Command) -> Vec<FieldDescriptor> {
    extract_fields_from_arguments(command.get_arguments())
}

/// Helper function to extract field descriptors from command arguments
fn extract_fields_from_arguments<'a>(
    args: impl Iterator<Item = &'a Arg> + 'a
) -> Vec<FieldDescriptor> {
    args
        .filter(|arg| {
            // Skip help and version arguments
            let id = arg.get_id().as_str();
            id != "help" && id != "version"
        })
        .map(|arg| {
            let name = arg.get_id().as_str().to_string();
            let short = arg.get_short().map(|c| c);
            let long = arg.get_long().map(|s| s.to_string());
            let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
            let is_positional = arg.is_positional();

            // Get default value
            let default_value = arg.get_default_values()
                .first()
                .and_then(|d| d.to_str().map(|s| s.to_string()));

            // Determine field type based on action and value parser
            let field_type = determine_field_type_from_arg(arg);

            // Determine if required
            let required = arg.is_required_set();

            FieldDescriptor {
                name,
                short,
                long,
                help,
                field_type,
                default_value,
                required,
                is_positional,
            }
        })
        .collect()
}

/// Extracts subcommand descriptors from a Clap Command
///
/// This function introspects a Clap Command to extract all subcommands
/// and their respective arguments.
///
/// # Arguments
///
/// * `command` - A Clap Command object
///
/// # Returns
///
/// A Vec of SubcommandDescriptor objects representing all subcommands
pub fn extract_subcommands_from_command(command: &Command) -> Vec<SubcommandDescriptor> {
    command
        .get_subcommands()
        .filter(|subcmd| {
            // Skip help subcommand
            subcmd.get_name() != "help"
        })
        .map(|subcmd| {
            let name = subcmd.get_name().to_string();
            let help = subcmd.get_about()
                .map(|a| a.to_string())
                .unwrap_or_default();
            let fields = extract_fields_from_arguments(subcmd.get_arguments());

            SubcommandDescriptor {
                name,
                help,
                fields,
            }
        })
        .collect()
}

fn determine_field_type_from_arg(arg: &Arg) -> FieldType {
    let action = arg.get_action();

    // Check action type first
    match action {
        ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Set if is_bool_arg(arg) => {
            return FieldType::Bool;
        }
        ArgAction::Count => {
            return FieldType::Counter;
        }
        ArgAction::Append => {
            return FieldType::Vec;
        }
        _ => {}
    }

    // Check if it takes multiple values
    let num_args = arg.get_num_args();
    if num_args.map(|n| n.max_values() > 1).unwrap_or(false) {
        return FieldType::Vec;
    }

    // Check if it's an enum (has possible values)
    if let Some(value_parser) = arg.get_value_parser().possible_values() {
        let options: Vec<EnumOption> = value_parser
            .map(|pv| EnumOption {
                value: pv.get_name().to_string(),
                help: pv.get_help().map(|h| h.to_string()).unwrap_or_default(),
            })
            .collect();
        if !options.is_empty() {
            return FieldType::Enum(options);
        }
    }

    // Try to infer from value parser type name
    let type_id = arg.get_value_parser().type_id();
    let type_name = format!("{:?}", type_id);

    if type_name.contains("bool") {
        return FieldType::Bool;
    }

    if type_name.contains("u8") || type_name.contains("u16") || type_name.contains("u32")
        || type_name.contains("u64") || type_name.contains("usize")
        || type_name.contains("i8") || type_name.contains("i16") || type_name.contains("i32")
        || type_name.contains("i64") || type_name.contains("isize") {
        return FieldType::Integer;
    }

    // Default to String
    FieldType::String
}

fn is_bool_arg(arg: &Arg) -> bool {
    // Check if the action suggests a boolean
    matches!(arg.get_action(), ArgAction::SetTrue | ArgAction::SetFalse)
        || arg.get_num_args().map(|n| n.takes_values()).unwrap_or(true) == false
}

/// Generates HTML for form fields based on field descriptors
///
/// # Arguments
/// * `fields` - The field descriptors to generate HTML for
/// * `prefix` - An optional prefix for field IDs (used for subcommand fields)
fn generate_form_fields_with_prefix(fields: &[FieldDescriptor], prefix: Option<&str>) -> Markup {
    html! {
        @for field in fields {
            @let id = if let Some(p) = prefix {
                format!("{}-{}", p, field.name)
            } else {
                field.name.clone()
            };

            // Use help text as label if available and not empty, otherwise use flag/name
            @let label_text = if !field.help.is_empty() {
                &field.help
            } else if field.is_positional {
                &field.name
            } else {
                field.long.as_ref().unwrap_or(&field.name)
            };

            // Show flag info as additional context (e.g., "-n, --name" or "--name")
            @let flag_info = if !field.is_positional {
                let mut parts = Vec::new();
                if let Some(s) = field.short {
                    parts.push(format!("-{}", s));
                }
                if let Some(ref l) = field.long {
                    parts.push(format!("--{}", l));
                }
                if !parts.is_empty() {
                    format!(" ({})", parts.join(", "))
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            @let required_marker = if field.required { " *" } else { "" };
            @let data_field_name = &field.name;
            @let data_is_positional = field.is_positional.to_string();

            @match &field.field_type {
                FieldType::String => {
                    @let default_val = field.default_value.as_deref().unwrap_or("");
                    // Use textarea for positional string arguments (no short/long flags)
                    @if field.short.is_none() && field.long.is_none() {
                        div.field-group
                            data-field-name=(data_field_name)
                            data-is-positional=(data_is_positional) {
                            label for=(id) { (label_text) (required_marker) }
                            textarea
                                  id=(id)
                                  name=(id)
                                  placeholder=(label_text)
                                  required[field.required]
                                  rows="5" { (default_val) }
                        }
                    } @else {
                        div.field-group
                            data-field-name=(data_field_name)
                            data-is-positional=(data_is_positional) {
                            label for=(id) { (label_text) (required_marker) }
                            @if !flag_info.is_empty() {
                                span.help-text { (flag_info) }
                            }
                            input type="text"
                                  id=(id)
                                  name=(id)
                                  value=(default_val)
                                  placeholder=(label_text)
                                  required[field.required];
                        }
                    }
                }
                FieldType::Bool => {
                    div.field-group.checkbox-group
                        data-field-name=(data_field_name)
                        data-is-positional=(data_is_positional) {
                        label for=(id) {
                            input type="checkbox" id=(id) name=(id);
                            (label_text) (required_marker)
                        }
                        @if !flag_info.is_empty() {
                            span.help-text { (flag_info) }
                        }
                    }
                }
                FieldType::Integer => {
                    @let default_val = field.default_value.as_deref().unwrap_or("0");
                    div.field-group
                        data-field-name=(data_field_name)
                        data-is-positional=(data_is_positional) {
                        label for=(id) { (label_text) (required_marker) }
                        @if !flag_info.is_empty() {
                            span.help-text { (flag_info) }
                        }
                        input type="number"
                              id=(id)
                              name=(id)
                              value=(default_val)
                              required[field.required];
                    }
                }
                FieldType::Counter => {
                    @let default_val = field.default_value.as_deref().unwrap_or("0");
                    div.field-group
                        data-field-name=(data_field_name)
                        data-is-positional=(data_is_positional) {
                        label for=(id) { (label_text) (required_marker) }
                        span.help-text { (flag_info) " (flag will be repeated N times)" }
                        input type="number"
                              id=(id)
                              name=(id)
                              value=(default_val)
                              min="0"
                              required[field.required];
                    }
                }
                FieldType::Enum(options) => {
                    @let default_val = field.default_value.as_deref().unwrap_or("");
                    div.field-group
                        data-field-name=(data_field_name)
                        data-is-positional=(data_is_positional) {
                        label for=(id) { (label_text) (required_marker) }
                        @if !flag_info.is_empty() {
                            span.help-text { (flag_info) }
                        }
                        select id=(id) name=(id) required[field.required] {
                            @if !field.required && default_val.is_empty() {
                                option value="" selected { "-- Select an option --" }
                            }
                            @for opt in options {
                                // Use help text if available, otherwise format the value name
                                @let display_text = if !opt.help.is_empty() {
                                    format!("{} ({})", opt.help, opt.value)
                                } else {
                                    // Format option display: capitalize and replace hyphens/underscores with spaces
                                    let s = opt.value.replace('-', " ").replace('_', " ");
                                    let mut c = s.chars();
                                    match c.next() {
                                        None => String::new(),
                                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                    }
                                };
                                @if &opt.value == default_val {
                                    option value=(&opt.value) selected { (display_text) }
                                } @else {
                                    option value=(&opt.value) { (display_text) }
                                }
                            }
                        }
                    }
                }
                FieldType::Vec => {
                    div.field-group.vec-group
                        data-field-name=(data_field_name)
                        data-is-positional=(data_is_positional)
                        data-vec-required=(field.required.to_string()) {
                        label for=(id) { (label_text) (required_marker) }
                        @if !flag_info.is_empty() {
                            span.help-text { (flag_info) }
                        }
                        div.vec-container id=(format!("{}-container", id)) {
                            input.vec-input
                                  type="text"
                                  placeholder="Enter value and press Enter"
                                  data-field-name=(id);
                            div.vec-items id=(format!("{}-items", id)) {}
                        }
                    }
                }
            }
        }
    }
}

/// Generates HTML for form fields (wrapper for backwards compatibility)
fn generate_form_fields(fields: &[FieldDescriptor]) -> Markup {
    generate_form_fields_with_prefix(fields, None)
}

/// Generates HTML for subcommand selector and fields
fn generate_subcommand_sections(subcommands: &[SubcommandDescriptor]) -> Markup {
    html! {
        @if !subcommands.is_empty() {
            div.form-section.subcommand-section {
                h2 { "Subcommands" }
                div.field-group {
                    label for="subcommand-selector" { "Select Subcommand" }
                    select #subcommand-selector name="subcommand" {
                        option value="" selected { "-- Select a subcommand --" }
                        @for subcmd in subcommands {
                            @let display_text = if !subcmd.help.is_empty() {
                                format!("{} ({})", subcmd.help, subcmd.name)
                            } else {
                                subcmd.name.clone()
                            };
                            option value=(&subcmd.name) { (display_text) }
                        }
                    }
                }

                @for subcmd in subcommands {
                    div.subcommand-fields
                        id=(format!("subcommand-{}", subcmd.name))
                        data-subcommand=(&subcmd.name)
                        style="display: none;" {
                        @let header_text = if !subcmd.help.is_empty() {
                            format!("{} ({})", subcmd.help, subcmd.name)
                        } else {
                            format!("Options for '{}'", subcmd.name)
                        };
                        h3 { (header_text) }
                        (generate_form_fields_with_prefix(&subcmd.fields, Some(&subcmd.name)))
                    }
                }
            }
        }
    }
}

/// Helper function to generate CSS styles
/// The CSS styles are loaded from cli-ui.css for better readability
fn generate_styles() -> Markup {
    // Load the CSS from the separate file at compile time
    const CSS_CONTENT: &str = include_str!("cli-ui.css");

    html! {
        style {
            (PreEscaped(CSS_CONTENT))
        }
    }
}

/// Helper function to generate JavaScript
/// The main JavaScript code is loaded from cli-ui.js for better readability
fn generate_script(function_name: &str, package_name: &str, fields_json: &str, subcommands_json: &str) -> Markup {
    // Load the JavaScript template from the separate file at compile time
    const JS_TEMPLATE: &str = include_str!("cli-ui.js");

    // Generate the configuration script (dynamic data only)
    let config_script = format!(
        r#"window.CLI_CONFIG = {{ fields: {}, subcommands: {} }};"#,
        fields_json,
        subcommands_json
    );

    // Convert package name to valid JavaScript module name (hyphens -> underscores)
    // wasm-pack converts package names like "rhyme-checker" to "rhyme_checker" in file names
    let js_package_name = package_name.replace('-', "_");

    // Replace placeholders in the JavaScript template with actual values
    // Since HTML is now in pkg/, import is relative to pkg/ directory
    let main_script = JS_TEMPLATE
        .replace("[FUNCTION_NAME]", function_name)
        .replace("[IMPORT_PATH]", &format!("./{}.js", js_package_name));

    html! {
        // First script: Set up configuration (inline)
        script {
            (PreEscaped(config_script))
        }
        // Second script: Main application logic (from cli-ui.js)
        script type="module" {
            (PreEscaped(main_script))
        }
    }
}

/// Generates a static HTML page for interacting with a WASM-bound Rust function
///
/// # Arguments
///
/// * `config` - Configuration specifying the WASM function details
///
/// # Returns
///
/// A String containing the complete HTML page
///
/// # Example
///
/// ```
/// use code_gen::{generate_wasm_function_page, WasmFunctionConfig, FieldDescriptor, FieldType};
///
/// let config = WasmFunctionConfig {
///     function_name: "process".to_string(),
///     package_name: "example".to_string(),
///     page_title: "My WASM Function".to_string(),
///     fields: vec![
///         FieldDescriptor {
///             name: "name".to_string(),
///             short: Some('n'),
///             long: Some("name".to_string()),
///             help: "Your name".to_string(),
///             field_type: FieldType::String,
///             default_value: None,
///             required: true,
///             is_positional: false,
///         }
///     ],
///     subcommands: vec![],
/// };
///
/// let html = generate_wasm_function_page(&config);
/// std::fs::write("output.html", html).unwrap();
/// ```
pub fn generate_wasm_function_page(config: &WasmFunctionConfig) -> String {
    let form_fields = generate_form_fields(&config.fields);
    let subcommand_sections = generate_subcommand_sections(&config.subcommands);
    let fields_json = serde_json::to_string(&config.fields).unwrap_or_else(|_| "[]".to_string());
    let subcommands_json = serde_json::to_string(&config.subcommands).unwrap_or_else(|_| "[]".to_string());

    let page = html! {
        (DOCTYPE)
        html {
            head {
                meta charset="UTF-8";
                title { (config.page_title) }
                (generate_styles())
            }
            body {
                div .container {
                    h1 { (config.page_title) }
                    div #status {}

                    form #cliForm {
                        div .form-section {
                            (form_fields)
                        }

                        (subcommand_sections)

                        div .button-group {
                            button #runButton type="button" { "Run" }
                            button #clearButton.clear-btn type="button" { "Reset" }
                        }
                    }

                    div .output-section {
                        label { "Output:" }
                        pre #output { "No output yet. Fill in the form and click \"Run\"." }
                    }
                }

                (generate_script(&config.function_name, &config.package_name, &fields_json, &subcommands_json))
            }
        }
    };

    page.into_string()
}

/// Simplified UI generation for Parser types
///
/// This function automatically extracts field information from a type that implements
/// both `clap::Parser` and `clap::CommandFactory`, eliminating the need to manually
/// construct `WasmFunctionConfig`.
///
/// # Type Parameters
///
/// * `T` - A type that implements both `Parser` and `CommandFactory` (typically a struct with `#[derive(Parser)]`)
///
/// # Arguments
///
/// * `package_name` - The package name (used in import path, e.g., "example" for "./example.js" when HTML is in pkg/)
/// * `page_title` - The title to display on the web page
///
/// # Returns
///
/// A String containing the complete HTML page
///
/// # Example
///
/// ```
/// use clap::Parser;
/// use code_gen::generate_ui_for_parser;
///
/// #[derive(Parser)]
/// struct MyArgs {
///     #[arg(short, long)]
///     name: String,
/// }
///
/// let html = generate_ui_for_parser::<MyArgs>("my_package", "My Web UI");
/// std::fs::write("ui.html", html).unwrap();
/// ```
pub fn generate_ui_for_parser<T: clap::Parser + clap::CommandFactory>(
    package_name: &str,
    page_title: &str,
) -> String {
    generate_ui_for_parser_with_function::<T>(package_name, page_title, "process_bind")
}

/// Simplified UI generation for Parser types with custom function name
///
/// Like `generate_ui_for_parser`, but allows specifying a custom WASM function name.
/// This is useful if your `#[web_ui_bind]` function has a different name than "process".
///
/// # Type Parameters
///
/// * `T` - A type that implements both `Parser` and `CommandFactory`
///
/// # Arguments
///
/// * `package_name` - The package name (used in import path)
/// * `page_title` - The title to display on the web page
/// * `function_name` - The name of the WASM-bound function (e.g., "process_bind" for `fn process`)
///
/// # Returns
///
/// A String containing the complete HTML page
///
/// # Example
///
/// ```
/// use clap::Parser;
/// use code_gen::generate_ui_for_parser_with_function;
///
/// #[derive(Parser)]
/// struct MyArgs {
///     #[arg(short, long)]
///     name: String,
/// }
///
/// // For a function named `execute` (which generates `execute_bind`)
/// let html = generate_ui_for_parser_with_function::<MyArgs>(
///     "my_package",
///     "My Web UI",
///     "execute_bind"
/// );
/// std::fs::write("ui.html", html).unwrap();
/// ```
pub fn generate_ui_for_parser_with_function<T: clap::Parser + clap::CommandFactory>(
    package_name: &str,
    page_title: &str,
    function_name: &str,
) -> String {
    let cmd = T::command();
    let fields = extract_field_descriptors_from_command(&cmd);
    let subcommands = extract_subcommands_from_command(&cmd);

    let config = WasmFunctionConfig {
        function_name: function_name.to_string(),
        package_name: package_name.to_string(),
        page_title: page_title.to_string(),
        fields,
        subcommands,
    };

    generate_wasm_function_page(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic_page() {
        let config = WasmFunctionConfig {
            function_name: "test_func".to_string(),
            package_name: "test_pkg".to_string(),
            page_title: "Test Page".to_string(),
            fields: vec![
                FieldDescriptor {
                    name: "test_field".to_string(),
                    short: Some('t'),
                    long: Some("test".to_string()),
                    help: "Test field".to_string(),
                    field_type: FieldType::String,
                    default_value: None,
                    required: false,
                    is_positional: false,
                }
            ],
            subcommands: vec![],
        };

        let html = generate_wasm_function_page(&config);

        assert!(html.contains("Test Page"));
        assert!(html.contains("test_func"));
        assert!(html.contains("./test_pkg.js"));
        assert!(html.contains("test_field"));
    }

    #[test]
    fn test_generate_page_with_fields() {
        let config = WasmFunctionConfig {
            function_name: "process".to_string(),
            package_name: "example".to_string(),
            page_title: "Example".to_string(),
            fields: vec![
                FieldDescriptor {
                    name: "name".to_string(),
                    short: Some('n'),
                    long: Some("name".to_string()),
                    help: "Name field".to_string(),
                    field_type: FieldType::String,
                    default_value: Some("default".to_string()),
                    required: true,
                    is_positional: false,
                },
                FieldDescriptor {
                    name: "enabled".to_string(),
                    short: Some('e'),
                    long: Some("enabled".to_string()),
                    help: "Enable feature".to_string(),
                    field_type: FieldType::Bool,
                    default_value: None,
                    required: false,
                    is_positional: false,
                },
            ],
            subcommands: vec![],
        };

        let html = generate_wasm_function_page(&config);

        assert!(html.contains("name"));
        assert!(html.contains("enabled"));
        assert!(html.contains("Name field"));
        assert!(html.contains("Enable feature"));
    }

    #[test]
    fn test_enum_field_generation() {
        let config = WasmFunctionConfig {
            function_name: "test".to_string(),
            package_name: "test".to_string(),
            page_title: "Test".to_string(),
            fields: vec![
                FieldDescriptor {
                    name: "color".to_string(),
                    short: Some('c'),
                    long: Some("color".to_string()),
                    help: "Select color".to_string(),
                    field_type: FieldType::Enum(vec![
                        EnumOption { value: "red".to_string(), help: "Red color".to_string() },
                        EnumOption { value: "green".to_string(), help: "Green color".to_string() },
                        EnumOption { value: "blue".to_string(), help: "Blue color".to_string() },
                    ]),
                    default_value: Some("red".to_string()),
                    required: false,
                    is_positional: false,
                },
            ],
            subcommands: vec![],
        };

        let html = generate_wasm_function_page(&config);

        assert!(html.contains("color"));
        assert!(html.contains("red"));
        assert!(html.contains("Red color (red)"));
        assert!(html.contains("Green color (green)"));
        assert!(html.contains("Blue color (blue)"));
        assert!(html.contains("<select"));
    }

    #[test]
    fn test_extract_field_descriptors() {
        use clap::{Parser, ValueEnum, CommandFactory};

        #[derive(Parser)]
        #[command(name = "test")]
        struct TestArgs {
            /// Name field
            #[arg(short, long)]
            name: String,

            /// Count field
            #[arg(short = 'c', long, default_value = "5")]
            count: u32,

            /// Enable flag
            #[arg(short, long)]
            enabled: bool,

            /// Color choice
            #[arg(short = 'o', long, value_enum)]
            color: TestColor,

            /// Tags
            #[arg(short, long)]
            tags: Vec<String>,
        }

        #[derive(Clone, Copy, ValueEnum)]
        enum TestColor {
            Red,
            Green,
            Blue,
        }

        let cmd = TestArgs::command();
        let fields = extract_field_descriptors_from_command(&cmd);

        // Should extract 5 fields (not counting help/version)
        assert_eq!(fields.len(), 5);

        // Check name field
        let name_field = fields.iter().find(|f| f.name == "name").unwrap();
        assert_eq!(name_field.short, Some('n'));
        assert_eq!(name_field.long, Some("name".to_string()));
        assert_eq!(name_field.help, "Name field");
        assert!(matches!(name_field.field_type, FieldType::String));

        // Check count field with default
        let count_field = fields.iter().find(|f| f.name == "count").unwrap();
        assert_eq!(count_field.short, Some('c'));
        assert_eq!(count_field.default_value, Some("5".to_string()));
        assert!(matches!(count_field.field_type, FieldType::Integer));

        // Check bool field
        let enabled_field = fields.iter().find(|f| f.name == "enabled").unwrap();
        assert!(matches!(enabled_field.field_type, FieldType::Bool));

        // Check enum field
        let color_field = fields.iter().find(|f| f.name == "color").unwrap();
        if let FieldType::Enum(options) = &color_field.field_type {
            assert_eq!(options.len(), 3);
            assert!(options.iter().any(|o| o.value == "red"));
            assert!(options.iter().any(|o| o.value == "green"));
            assert!(options.iter().any(|o| o.value == "blue"));
        } else {
            panic!("Expected Enum field type");
        }

        // Check vec field
        let _tags_field = fields.iter().find(|f| f.name == "tags").unwrap();

    }
}


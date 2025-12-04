// Re-export the procedural macros
pub use code_gen_macro::{web_ui_bind, wprintln};

// Re-export paste for use in macros
#[doc(hidden)]
pub use paste;

use serde::Serialize;
use clap::{Command, Arg, ArgAction};

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
    Enum(Vec<String>),
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
}

/// Configuration for generating a WASM function web interface
pub struct WasmFunctionConfig {
    /// The name of the WASM function to call (e.g., "process")
    pub function_name: String,
    /// The package name (used in import path, e.g., "example" for "./pkg/example.js")
    pub package_name: String,
    /// The title to display on the web page
    pub page_title: String,
    /// Field descriptors for generating form inputs
    pub fields: Vec<FieldDescriptor>,
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
    command
        .get_arguments()
        .filter(|arg| {
            // Skip help and version arguments
            let id = arg.get_id().as_str();
            id != "help" && id != "version"
        })
        .filter(|arg| {
            // Skip positional arguments and subcommands for now
            !arg.is_positional()
        })
        .map(|arg| {
            let name = arg.get_id().as_str().to_string();
            let short = arg.get_short().map(|c| c);
            let long = arg.get_long().map(|s| s.to_string());
            let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();

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
        let values: Vec<String> = value_parser
            .map(|pv| pv.get_name().to_string())
            .collect();
        if !values.is_empty() {
            return FieldType::Enum(values);
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
fn generate_form_fields(fields: &[FieldDescriptor]) -> String {
    fields
        .iter()
        .map(|field| {
            let id = &field.name;
            let label_text = field.long.as_ref().unwrap_or(&field.name);
            let help = &field.help;
            let required = if field.required { " *" } else { "" };

            match &field.field_type {
                FieldType::String => {
                    let default_val = field.default_value.as_deref().unwrap_or("");
                    format!(
                        r#"<div class="field-group">
                <label for="{id}">{label_text}{required}</label>
                <span class="help-text">{help}</span>
                <input type="text" id="{id}" value="{default_val}" placeholder="Enter {label_text}">
            </div>"#
                    )
                }
                FieldType::Bool => {
                    format!(
                        r#"<div class="field-group checkbox-group">
                <label for="{id}">
                    <input type="checkbox" id="{id}">
                    {label_text}{required}
                </label>
                <span class="help-text">{help}</span>
            </div>"#
                    )
                }
                FieldType::Integer => {
                    let default_val = field.default_value.as_deref().unwrap_or("0");
                    format!(
                        r#"<div class="field-group">
                <label for="{id}">{label_text}{required}</label>
                <span class="help-text">{help}</span>
                <input type="number" id="{id}" value="{default_val}">
            </div>"#
                    )
                }
                FieldType::Counter => {
                    let default_val = field.default_value.as_deref().unwrap_or("0");
                    format!(
                        r#"<div class="field-group">
                <label for="{id}">{label_text}{required}</label>
                <span class="help-text">{help} (flag will be repeated N times)</span>
                <input type="number" id="{id}" value="{default_val}" min="0">
            </div>"#
                    )
                }
                FieldType::Enum(options) => {
                    let default_val = field.default_value.as_deref().unwrap_or("");
                    let options_html: String = options
                        .iter()
                        .map(|opt| {
                            let selected = if opt == default_val { " selected" } else { "" };
                            format!(r#"<option value="{opt}"{selected}>{opt}</option>"#)
                        })
                        .collect();
                    format!(
                        r#"<div class="field-group">
                <label for="{id}">{label_text}{required}</label>
                <span class="help-text">{help}</span>
                <select id="{id}">
                    {options_html}
                </select>
            </div>"#
                    )
                }
                FieldType::Vec => {
                    format!(
                        r#"<div class="field-group vec-group">
                <label for="{id}">{label_text}{required}</label>
                <span class="help-text">{help}</span>
                <div class="vec-container" id="{id}-container">
                    <input type="text" class="vec-input" placeholder="Enter value and press Enter">
                    <div class="vec-items" id="{id}-items"></div>
                </div>
            </div>"#
                    )
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n            ")
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
///         }
///     ],
/// };
///
/// let html = generate_wasm_function_page(&config);
/// std::fs::write("output.html", html).unwrap();
/// ```
pub fn generate_wasm_function_page(config: &WasmFunctionConfig) -> String {
    let form_fields = generate_form_fields(&config.fields);
    let fields_json = serde_json::to_string(&config.fields).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{page_title}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            max-width: 1000px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            background: white;
            border-radius: 8px;
            padding: 30px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-top: 0;
        }}
        .form-section {{
            margin: 20px 0;
        }}
        .field-group {{
            margin: 15px 0;
        }}
        .field-group label {{
            display: block;
            margin-bottom: 5px;
            font-weight: 600;
            color: #555;
        }}
        .help-text {{
            display: block;
            font-size: 0.9em;
            color: #888;
            margin-bottom: 5px;
        }}
        input[type="text"],
        input[type="number"],
        select {{
            width: 100%;
            padding: 8px 12px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 14px;
            box-sizing: border-box;
        }}
        input[type="text"]:focus,
        input[type="number"]:focus,
        select:focus {{
            outline: none;
            border-color: #4CAF50;
            box-shadow: 0 0 0 2px rgba(76,175,80,0.2);
        }}
        input:invalid {{
            border-color: #f44336;
        }}
        input.error {{
            border-color: #f44336;
            background-color: #ffebee;
        }}
        .checkbox-group {{
            display: flex;
            align-items: center;
        }}
        .checkbox-group label {{
            display: flex;
            align-items: center;
            cursor: pointer;
        }}
        .checkbox-group input[type="checkbox"] {{
            margin-right: 8px;
            width: auto;
        }}
        .vec-container {{
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 10px;
            background-color: #fafafa;
        }}
        .vec-input {{
            width: 100%;
            margin-bottom: 10px;
        }}
        .vec-items {{
            display: flex;
            flex-wrap: wrap;
            gap: 8px;
        }}
        .vec-item {{
            background-color: #4CAF50;
            color: white;
            padding: 5px 10px;
            border-radius: 4px;
            display: inline-flex;
            align-items: center;
            gap: 5px;
        }}
        .vec-item-remove {{
            cursor: pointer;
            font-weight: bold;
            padding: 0 5px;
        }}
        .button-group {{
            margin: 20px 0;
            display: flex;
            gap: 10px;
        }}
        button {{
            background-color: #4CAF50;
            color: white;
            border: none;
            padding: 12px 24px;
            font-size: 16px;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 500;
            transition: background-color 0.2s;
        }}
        button:hover {{
            background-color: #45a049;
        }}
        button:disabled {{
            background-color: #cccccc;
            cursor: not-allowed;
        }}
        .clear-btn {{
            background-color: #f44336;
        }}
        .clear-btn:hover {{
            background-color: #da190b;
        }}
        .output-section {{
            margin: 20px 0;
        }}
        pre {{
            background-color: #f8f8f8;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 15px;
            overflow-x: auto;
            min-height: 100px;
            white-space: pre-wrap;
            word-wrap: break-word;
        }}
        .error {{
            color: #f44336;
            background-color: #ffebee;
            border-color: #f44336;
        }}
        .success {{
            color: #4CAF50;
            background-color: #e8f5e9;
            border-color: #4CAF50;
        }}
        .status {{
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
            font-weight: 500;
        }}
        .loading {{
            color: #2196F3;
            background-color: #e3f2fd;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{page_title}</h1>

        <div id="status"></div>

        <div class="form-section">
            {form_fields}
        </div>

        <div class="button-group">
            <button id="runButton">Run Function</button>
            <button id="clearButton" class="clear-btn">Clear All</button>
        </div>

        <div class="output-section">
            <label>Output:</label>
            <pre id="output">No output yet. Fill in the form and click "Run Function".</pre>
        </div>
    </div>

    <script type="module">
        import init, {{ {function_name} }} from './pkg/{package_name}.js';

        let wasmReady = false;

        // Field metadata
        const FIELDS = {fields_json};

        function setStatus(message, type) {{
            const statusDiv = document.getElementById('status');
            statusDiv.textContent = message;
            statusDiv.className = 'status ' + type;
            if (!message) {{
                statusDiv.style.display = 'none';
            }} else {{
                statusDiv.style.display = 'block';
            }}
        }}

        async function initWasm() {{
            try {{
                setStatus('Loading WASM module...', 'loading');
                await init();
                wasmReady = true;
                setStatus('WASM module loaded successfully!', 'success');
                setTimeout(() => setStatus('', ''), 2000);
                console.log('WASM module loaded successfully');
            }} catch (e) {{
                setStatus('Failed to load WASM module: ' + e, 'error');
                console.error('Failed to load WASM module:', e);
            }}
        }}

        // Initialize vec field handlers
        function initVecFields() {{
            FIELDS.forEach((field) => {{
                if (field.field_type.type === 'Vec') {{
                    const container = document.getElementById(field.name + '-container');
                    if (container) {{
                        const input = container.querySelector('.vec-input');
                        const items = document.getElementById(field.name + '-items');

                        input.addEventListener('keydown', (e) => {{
                            if (e.key === 'Enter' && input.value.trim()) {{
                                e.preventDefault();
                                addVecItem(field.name, input.value.trim());
                                input.value = '';
                            }}
                        }});
                    }}
                }}
            }});
        }}

        function addVecItem(fieldName, value) {{
            const items = document.getElementById(fieldName + '-items');
            const item = document.createElement('div');
            item.className = 'vec-item';
            item.innerHTML = `${{value}}<span class="vec-item-remove" onclick="this.parentElement.remove()">\u00d7</span>`;
            items.appendChild(item);
        }}

        function getVecValues(fieldName) {{
            const items = document.getElementById(fieldName + '-items');
            return Array.from(items.children).map(item => {{
                return item.textContent.slice(0, -1); // Remove the × character
            }});
        }}

        function validateFields() {{
            const errors = [];

            // Clear previous error styling
            FIELDS.forEach((field) => {{
                const element = document.getElementById(field.name);
                if (element) {{
                    element.classList.remove('error');
                }}
            }});

            FIELDS.forEach((field) => {{
                const element = document.getElementById(field.name);
                if (!element) return;

                const fieldTypeName = field.field_type.type;
                const fieldLabel = field.long || field.name;
                let hasError = false;

                // Check HTML5 validation state for number inputs
                if (fieldTypeName === 'Integer' || fieldTypeName === 'Counter') {{
                    if (element.validity && !element.validity.valid) {{
                        errors.push(`Field "${{fieldLabel}}": Invalid number value`);
                        hasError = true;
                    }}
                }}

                // Check required fields
                if (field.required) {{
                    if (fieldTypeName === 'Bool') {{
                        // Bools can't really be required in the traditional sense
                    }} else if (fieldTypeName === 'Vec') {{
                        const values = getVecValues(field.name);
                        if (values.length === 0) {{
                            errors.push(`Field "${{fieldLabel}}": Required field is empty`);
                            hasError = true;
                        }}
                    }} else {{
                        const value = element.value.trim();
                        if (!value) {{
                            errors.push(`Field "${{fieldLabel}}": Required field is empty`);
                            hasError = true;
                        }}
                    }}
                }}

                // Add error styling if field has error
                if (hasError) {{
                    element.classList.add('error');
                }}
            }});

            return errors;
        }}

        function formToCliArgs() {{
            const args = [];

            FIELDS.forEach((field) => {{
                const element = document.getElementById(field.name);
                if (!element) return;

                const fieldTypeName = field.field_type.type;
                const short = field.short;
                const long = field.long;

                if (fieldTypeName === 'Bool') {{
                    if (element.checked) {{
                        args.push(long ? `--${{long}}` : `-${{short}}`);
                    }}
                }} else if (fieldTypeName === 'Counter') {{
                    // Counter fields: repeat the flag N times without values
                    const count = parseInt(element.value) || 0;
                    const flag = long ? `--${{long}}` : `-${{short}}`;
                    for (let i = 0; i < count; i++) {{
                        args.push(flag);
                    }}
                }} else if (fieldTypeName === 'Vec') {{
                    const values = getVecValues(field.name);
                    values.forEach(value => {{
                        if (long) {{
                            args.push(`--${{long}}`, value);
                        }} else if (short) {{
                            args.push(`-${{short}}`, value);
                        }}
                    }});
                }} else {{
                    // String, Integer, Enum - pass value as argument
                    const value = element.value.trim();
                    if (value) {{
                        if (long) {{
                            args.push(`--${{long}}`, value);
                        }} else if (short) {{
                            args.push(`-${{short}}`, value);
                        }}
                    }}
                }}
            }});

            return args;
        }}

        function runFunction() {{
            if (!wasmReady) {{
                setStatus('WASM module not ready yet. Please wait...', 'error');
                return;
            }}

            const outputElement = document.getElementById('output');
            const runButton = document.getElementById('runButton');

            try {{
                // Validate fields first
                const validationErrors = validateFields();
                if (validationErrors.length > 0) {{
                    outputElement.className = 'error';
                    outputElement.textContent = 'Validation Error:\n' + validationErrors.join('\n');
                    setStatus('Please fix validation errors', 'error');
                    return;
                }}

                // Convert form to CLI args
                const args = formToCliArgs();
                console.log('CLI args:', args);

                // Disable button during execution
                runButton.disabled = true;
                setStatus('Running function...', 'loading');

                // Call the WASM function with string array
                const result = {function_name}(args);

                // Display result
                outputElement.className = 'success';
                if (result !== undefined && result !== null) {{
                    if (typeof result === 'string') {{
                        outputElement.textContent = result;
                    }} else {{
                        outputElement.textContent = JSON.stringify(result, null, 2);
                    }}
                }} else {{
                    outputElement.textContent = 'Function executed successfully (no return value)';
                }}
                setStatus('Function executed successfully!', 'success');
                setTimeout(() => setStatus('', ''), 2000);

            }} catch (e) {{
                outputElement.className = 'error';
                outputElement.textContent = 'Error:\n' + e;
                setStatus('Error occurred', 'error');
            }} finally {{
                runButton.disabled = false;
            }}
        }}

        function clearAll() {{
            FIELDS.forEach((field) => {{
                const element = document.getElementById(field.name);
                if (!element) return;

                const fieldTypeName = field.field_type.type;

                if (fieldTypeName === 'Bool') {{
                    element.checked = false;
                }} else if (fieldTypeName === 'Vec') {{
                    const items = document.getElementById(field.name + '-items');
                    items.innerHTML = '';
                }} else {{
                    element.value = '';
                }}
            }});
            document.getElementById('output').textContent = 'No output yet. Fill in the form and click "Run Function".';
            document.getElementById('output').className = '';
            setStatus('', '');
        }}

        // Add event listeners
        document.getElementById('runButton').addEventListener('click', runFunction);
        document.getElementById('clearButton').addEventListener('click', clearAll);

        // Initialize WASM on page load
        initWasm();
        initVecFields();
    </script>
</body>
</html>"#,
        page_title = config.page_title,
        function_name = config.function_name,
        package_name = config.package_name,
        form_fields = form_fields,
        fields_json = fields_json
    )
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
/// * `package_name` - The package name (used in import path, e.g., "example" for "./pkg/example.js")
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

    let config = WasmFunctionConfig {
        function_name: function_name.to_string(),
        package_name: package_name.to_string(),
        page_title: page_title.to_string(),
        fields,
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
                }
            ],
        };

        let html = generate_wasm_function_page(&config);

        assert!(html.contains("Test Page"));
        assert!(html.contains("test_func"));
        assert!(html.contains("./pkg/test_pkg.js"));
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
                },
                FieldDescriptor {
                    name: "enabled".to_string(),
                    short: Some('e'),
                    long: Some("enabled".to_string()),
                    help: "Enable feature".to_string(),
                    field_type: FieldType::Bool,
                    default_value: None,
                    required: false,
                },
            ],
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
                        "red".to_string(),
                        "green".to_string(),
                        "blue".to_string(),
                    ]),
                    default_value: Some("red".to_string()),
                    required: false,
                },
            ],
        };

        let html = generate_wasm_function_page(&config);

        assert!(html.contains("color"));
        assert!(html.contains("red"));
        assert!(html.contains("green"));
        assert!(html.contains("blue"));
        assert!(html.contains("<select"));
    }

    #[test]
    fn test_extract_field_descriptors() {
        use clap::{Parser, ValueEnum, CommandFactory};

        #[derive(Parser)]
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

        #[derive(Clone, ValueEnum)]
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
        if let FieldType::Enum(values) = &color_field.field_type {
            assert_eq!(values.len(), 3);
            assert!(values.contains(&"red".to_string()));
            assert!(values.contains(&"green".to_string()));
            assert!(values.contains(&"blue".to_string()));
        } else {
            panic!("Expected Enum field type");
        }

        // Check vec field
        let tags_field = fields.iter().find(|f| f.name == "tags").unwrap();
        assert!(matches!(tags_field.field_type, FieldType::Vec));
    }
}

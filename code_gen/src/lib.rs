// Re-export the procedural macro
pub use code_gen_macro::web_ui_bind;

// Re-export paste for use in macros
#[doc(hidden)]
pub use paste;


/// Configuration for generating a WASM function web interface
pub struct WasmFunctionConfig {
    /// The name of the WASM function to call (e.g., "process")
    pub function_name: String,
    /// The package name (used in import path, e.g., "example" for "./pkg/example.js")
    pub package_name: String,
    /// The title to display on the web page
    pub page_title: String,
    /// Optional example JSON to pre-fill the input
    pub example_json: Option<String>,
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
/// use code_gen::{generate_wasm_function_page, WasmFunctionConfig};
///
/// let config = WasmFunctionConfig {
///     function_name: "process".to_string(),
///     package_name: "example".to_string(),
///     page_title: "My WASM Function".to_string(),
///     example_json: Some(r#"{"field": "value"}"#.to_string()),
/// };
///
/// let html = generate_wasm_function_page(&config);
/// std::fs::write("output.html", html).unwrap();
/// ```
pub fn generate_wasm_function_page(config: &WasmFunctionConfig) -> String {
    let example_json = config.example_json.as_deref().unwrap_or("{}");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{page_title}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            max-width: 1200px;
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
        .input-section {{
            margin: 20px 0;
        }}
        label {{
            display: block;
            margin-bottom: 8px;
            font-weight: 600;
            color: #555;
        }}
        textarea {{
            width: 100%;
            min-height: 200px;
            padding: 12px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-family: 'Courier New', monospace;
            font-size: 14px;
            box-sizing: border-box;
            resize: vertical;
        }}
        textarea:focus {{
            outline: none;
            border-color: #4CAF50;
            box-shadow: 0 0 0 2px rgba(76,175,80,0.2);
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
        button:active {{
            background-color: #3d8b40;
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

        <div class="input-section">
            <label for="jsonInput">Function Parameters (JSON):</label>
            <textarea id="jsonInput" placeholder="Enter JSON parameters here...">{example_json}</textarea>
        </div>

        <div class="button-group">
            <button id="runButton">Run Function</button>
            <button id="clearButton" class="clear-btn">Clear All</button>
        </div>

        <div class="output-section">
            <label>Output:</label>
            <pre id="output">No output yet. Enter JSON parameters and click "Run Function".</pre>
        </div>
    </div>

    <script type="module">
        import init, {{ {function_name} }} from './pkg/{package_name}.js';

        let wasmReady = false;

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

        function runFunction() {{
            if (!wasmReady) {{
                setStatus('WASM module not ready yet. Please wait...', 'error');
                return;
            }}

            const inputElement = document.getElementById('jsonInput');
            const outputElement = document.getElementById('output');
            const runButton = document.getElementById('runButton');

            try {{
                // Parse JSON input
                const jsonText = inputElement.value.trim();
                if (!jsonText) {{
                    throw new Error('Please enter JSON parameters');
                }}

                const params = JSON.parse(jsonText);

                // Disable button during execution
                runButton.disabled = true;
                setStatus('Running function...', 'loading');

                // Call the WASM function
                const result = {function_name}(params);

                // Display result
                outputElement.className = 'success';
                if (result !== undefined && result !== null) {{
                    // If result is a string, display it directly; otherwise JSON stringify
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
                if (e instanceof SyntaxError) {{
                    outputElement.textContent = 'JSON Parse Error:\n' + e.message + '\n\nPlease check your JSON syntax.';
                }} else {{
                    outputElement.textContent = 'Error:\n' + e;
                }}
                setStatus('Error occurred', 'error');
            }} finally {{
                runButton.disabled = false;
            }}
        }}

        function clearAll() {{
            document.getElementById('jsonInput').value = '';
            document.getElementById('output').textContent = 'No output yet. Enter JSON parameters and click "Run Function".';
            document.getElementById('output').className = '';
            setStatus('', '');
        }}

        // Add event listeners
        document.getElementById('runButton').addEventListener('click', runFunction);
        document.getElementById('clearButton').addEventListener('click', clearAll);

        // Handle Enter key in textarea (Ctrl+Enter to run)
        document.getElementById('jsonInput').addEventListener('keydown', function(e) {{
            if (e.ctrlKey && e.key === 'Enter') {{
                runFunction();
            }}
        }});

        // Initialize WASM on page load
        initWasm();
    </script>
</body>
</html>"#,
        page_title = config.page_title,
        function_name = config.function_name,
        package_name = config.package_name,
        example_json = example_json
    )
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
            example_json: None,
        };

        let html = generate_wasm_function_page(&config);

        assert!(html.contains("Test Page"));
        assert!(html.contains("test_func"));
        assert!(html.contains("./pkg/test_pkg.js"));
    }

    #[test]
    fn test_generate_page_with_example() {
        let config = WasmFunctionConfig {
            function_name: "process".to_string(),
            package_name: "example".to_string(),
            page_title: "Example".to_string(),
            example_json: Some(r#"{"field": "value"}"#.to_string()),
        };

        let html = generate_wasm_function_page(&config);

        assert!(html.contains(r#"{"field": "value"}"#));
    }
}

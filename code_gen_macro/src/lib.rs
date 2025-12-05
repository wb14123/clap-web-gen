use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// wprintln! - Web println! that captures output in WASM builds
#[proc_macro]
pub fn wprintln(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let expanded = quote! {
        {
            #[cfg(target_arch = "wasm32")]
            {
                __web_ui_capture::write_fmt(format_args!(#input));
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::println!(#input);
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn web_ui_bind(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_output = &input_fn.sig.output;

    // Extract parameter name and type
    let param = input_fn.sig.inputs.first().expect("Function must have at least one parameter");
    let (param_name, param_type) = if let syn::FnArg::Typed(pat_type) = param {
        let param_name = if let syn::Pat::Ident(ident) = &*pat_type.pat {
            &ident.ident
        } else {
            panic!("Parameter must be a simple identifier");
        };

        // Extract the inner type from &Type
        let inner_type = if let syn::Type::Reference(type_ref) = &*pat_type.ty {
            &type_ref.elem
        } else {
            panic!("Parameter must be a reference");
        };

        (param_name, inner_type)
    } else {
        panic!("Function must have typed parameters");
    };

    let bind_fn_name = syn::Ident::new(&format!("{}_bind", fn_name), fn_name.span());
    let ui_gen_fn_name = syn::Ident::new(&format!("generate_{}_ui", fn_name), fn_name.span());

    // Use a fixed module name since we want one println! override for the whole module
    let capture_mod_name = syn::Ident::new("__web_ui_capture", fn_name.span());

    // Convert bind_fn_name to string literal for use in the generated code
    let bind_fn_name_str = bind_fn_name.to_string();

    // Check if the function returns a Result
    let returns_result = matches!(fn_output, syn::ReturnType::Type(_, ty)
        if matches!(&**ty, syn::Type::Path(type_path)
            if type_path.path.segments.last()
                .map(|seg| seg.ident == "Result")
                .unwrap_or(false)));

    // Generate the appropriate capture call based on return type
    let capture_call = if returns_result {
        quote! {
            #capture_mod_name::capture_result(|| #fn_name(&#param_name))
                .map_err(|e| wasm_bindgen::prelude::JsValue::from_str(&format!("{:?}", e)))
        }
    } else {
        quote! {
            Ok(#capture_mod_name::capture(|| #fn_name(&#param_name)))
        }
    };

    let expanded = quote! {
        // Generate the capture infrastructure
        #[cfg(target_arch = "wasm32")]
        #[allow(dead_code)]
        mod __web_ui_capture {
            use std::cell::RefCell;
            use std::fmt::Write;

            thread_local! {
                pub static BUFFER: RefCell<String> = RefCell::new(String::new());
            }

            pub fn capture<F: FnOnce()>(f: F) -> String {
                BUFFER.with(|buf| buf.borrow_mut().clear());
                f();
                BUFFER.with(|buf| buf.borrow().clone())
            }

            pub fn capture_result<F, E>(f: F) -> Result<String, E>
            where
                F: FnOnce() -> Result<(), E>,
            {
                BUFFER.with(|buf| buf.borrow_mut().clear());
                f()?;
                Ok(BUFFER.with(|buf| buf.borrow().clone()))
            }

            pub fn write_fmt(args: std::fmt::Arguments) {
                BUFFER.with(|buf| {
                    let _ = writeln!(buf.borrow_mut(), "{}", args);
                });
            }
        }

        // Original function (unchanged)
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#param_name: &#param_type) #fn_output #fn_block

        // WASM binding function that uses the __web_ui_capture module
        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn #bind_fn_name(
            args: Vec<String>
        ) -> Result<String, wasm_bindgen::prelude::JsValue> {
            // Prepend program name (required by clap)
            let mut cli_args = vec!["program".to_string()];
            cli_args.extend(args);

            let #param_name = <#param_type as clap::Parser>::try_parse_from(&cli_args)
                .map_err(|e| wasm_bindgen::prelude::JsValue::from_str(&e.to_string()))?;

            #capture_call
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub fn #bind_fn_name(_opt: ()) -> Result<String, String> {
            Ok("WASM binding only available in wasm32 builds".to_string())
        }

        // Auto-generated UI generation function
        /// Generates a web UI HTML page for this function
        ///
        /// This function is automatically generated by the `#[web_ui_bind]` macro.
        /// It creates a complete HTML page that allows users to interact with the
        /// WASM-bound function through a web interface.
        ///
        /// # Arguments
        ///
        /// * `package_name` - The package name (used in import path, e.g., "example" for "./pkg/example.js")
        /// * `page_title` - The title to display on the web page
        ///
        /// # Returns
        ///
        /// A String containing the complete HTML page
        pub fn #ui_gen_fn_name(package_name: &str, page_title: &str) -> String {
            use clap::CommandFactory;

            let cmd = <#param_type as clap::CommandFactory>::command();
            let fields = code_gen::extract_field_descriptors_from_command(&cmd);
            let subcommands = code_gen::extract_subcommands_from_command(&cmd);

            let config = code_gen::WasmFunctionConfig {
                function_name: #bind_fn_name_str.to_string(),
                package_name: package_name.to_string(),
                page_title: page_title.to_string(),
                fields,
                subcommands,
            };

            code_gen::generate_wasm_function_page(&config)
        }
    };

    TokenStream::from(expanded)
}

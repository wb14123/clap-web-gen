use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn web_ui_setup(_item: TokenStream) -> TokenStream {
    let expanded = quote! {
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

            pub fn write_fmt(args: std::fmt::Arguments) {
                BUFFER.with(|buf| {
                    let _ = writeln!(buf.borrow_mut(), "{}", args);
                });
            }
        }

        // wprintln! - web println! for capturing output
        #[allow(unused_macros)]
        macro_rules! wprintln {
            () => {
                {
                    #[cfg(target_arch = "wasm32")]
                    {
                        __web_ui_capture::write_fmt(format_args!(""));
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        std::println!();
                    }
                }
            };
            ($($arg:tt)*) => {
                {
                    #[cfg(target_arch = "wasm32")]
                    {
                        __web_ui_capture::write_fmt(format_args!($($arg)*));
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        std::println!($($arg)*);
                    }
                }
            };
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

    // Use a fixed module name since we want one println! override for the whole module
    let capture_mod_name = syn::Ident::new("__web_ui_capture", fn_name.span());

    let expanded = quote! {
        // Original function (unchanged)
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#param_name: &#param_type) #fn_block

        // WASM binding function that uses the __web_ui_capture module
        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn #bind_fn_name(
            opt: wasm_bindgen::prelude::JsValue
        ) -> Result<String, wasm_bindgen::prelude::JsValue> {
            let #param_name: #param_type = serde_wasm_bindgen::from_value(opt)
                .map_err(|e| wasm_bindgen::prelude::JsValue::from_str(
                    &format!("Failed to parse {}: {:?}", stringify!(#param_type), e)
                ))?;

            Ok(#capture_mod_name::capture(|| #fn_name(&#param_name)))
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub fn #bind_fn_name(_opt: ()) -> Result<String, String> {
            Ok("WASM binding only available in wasm32 builds".to_string())
        }
    };

    TokenStream::from(expanded)
}

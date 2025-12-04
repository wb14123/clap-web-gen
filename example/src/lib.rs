use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;

/// A CLI tool demonstrating various Clap features
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "example")]
#[command(author = "Example Author <author@example.com>")]
#[command(version = "1.0")]
#[command(about = "Example CLI with various Clap features", long_about = None)]
pub struct Opt {
    /// Optional string field
    #[arg(short = 's', long)]
    pub string_field: Option<String>,

    /// String with default value
    #[arg(short = 'd', long, default_value = "default.txt")]
    pub string_default: String,

    /// Counter field (can be used multiple times: -c, -cc, -ccc)
    #[arg(short = 'c', long, action = clap::ArgAction::Count)]
    pub counter_field: u8,

    /// Boolean flag field
    #[arg(short = 'b', long)]
    pub bool_field: bool,

    /// Integer field with default
    #[arg(short = 'i', long, default_value = "42")]
    pub int_field: u64,

    /// Enum field
    #[arg(short = 'e', long, value_enum, default_value = "option-a")]
    pub enum_field: EnumType,

    /// Vec field (can be specified multiple times)
    #[arg(short = 'v', long)]
    pub vec_field: Vec<String>,

    /// Unsigned int field
    #[arg(short = 'u', long, default_value = "10")]
    pub uint_field: usize,

    /// Another optional string
    #[arg(short = 'o', long)]
    pub optional_field: Option<String>,

    /// Another boolean flag
    #[arg(short = 'f', long)]
    pub flag_field: bool,

    #[command(subcommand)]
    pub subcommand: Option<SubCommands>,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum EnumType {
    OptionA,
    OptionB,
    OptionC,
    OptionD,
}

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
pub enum SubCommands {
    /// First subcommand with string and bool
    Sub1 {
        /// Positional string argument
        #[arg(required = true)]
        arg1: String,

        /// Boolean flag
        #[arg(short, long)]
        flag1: bool,
    },

    /// Second subcommand with string and int
    Sub2 {
        /// String option
        #[arg(short, long)]
        str_arg: String,

        /// Integer option with default
        #[arg(short, long, default_value = "5")]
        num_arg: usize,
    },

    /// Third subcommand with positional and flag
    Sub3 {
        /// Positional argument
        pos_arg: String,

        /// Flag option
        #[arg(short, long)]
        flag_arg: bool,
    },
}

// Custom println! for WASM that captures output
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static OUTPUT_BUFFER: RefCell<String> = RefCell::new(String::new());
}

#[cfg(target_arch = "wasm32")]
macro_rules! println {
    () => {
        OUTPUT_BUFFER.with(|buf| buf.borrow_mut().push('\n'))
    };
    ($($arg:tt)*) => {{
        OUTPUT_BUFFER.with(|buf| {
            use std::fmt::Write;
            let _ = writeln!(buf.borrow_mut(), $($arg)*);
        })
    }};
}

#[cfg(target_arch = "wasm32")]
fn capture_output<F: FnOnce()>(f: F) -> String {
    OUTPUT_BUFFER.with(|buf| buf.borrow_mut().clear());
    f();
    OUTPUT_BUFFER.with(|buf| buf.borrow().clone())
}

pub fn process(opt: &Opt) {
    println!("Processing with options:");
    println!("  string_field: {:?}", opt.string_field);
    println!("  string_default: {}", opt.string_default);
    println!("  counter_field: {}", opt.counter_field);
    println!("  bool_field: {}", opt.bool_field);
    println!("  int_field: {}", opt.int_field);
    println!("  enum_field: {:?}", opt.enum_field);
    println!("  vec_field: {:?}", opt.vec_field);
    println!("  uint_field: {}", opt.uint_field);
    println!("  optional_field: {:?}", opt.optional_field);
    println!("  flag_field: {}", opt.flag_field);

    if let Some(cmd) = &opt.subcommand {
        println!("\nExecuting subcommand:");
        match cmd {
            SubCommands::Sub1 { arg1, flag1 } => {
                println!("  Sub1: arg1='{}', flag1={}", arg1, flag1);
            }
            SubCommands::Sub2 { str_arg, num_arg } => {
                println!("  Sub2: str_arg='{}', num_arg={}", str_arg, num_arg);
            }
            SubCommands::Sub3 { pos_arg, flag_arg } => {
                println!("  Sub3: pos_arg='{}', flag_arg={}", pos_arg, flag_arg);
            }
        }
    }
}

#[wasm_bindgen]
pub fn process_bind(opt: JsValue) -> Result<String, JsValue> {
    let opt: Opt = from_value(opt)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse Opt: {:?}", e)))?;

    #[cfg(target_arch = "wasm32")]
    {
        Ok(capture_output(|| process(&opt)))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        process(&opt);
        Ok("Output printed to console".to_string())
    }
}
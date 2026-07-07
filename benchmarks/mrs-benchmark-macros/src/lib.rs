use proc_macro::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use syn::{parse_macro_input, DeriveInput, LitStr};

#[derive(Deserialize, Debug)]
struct BenchConfig {
    cases: Vec<BenchCase>,
}

#[derive(Deserialize, Debug)]
struct BenchCase {
    test: String,
    repetitions: u32,
    inputs: Vec<serde_json::Value>,
    platforms: Option<Vec<String>>,
    libraries: Vec<String>,
}

fn load_config() -> BenchConfig {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut path = PathBuf::from(manifest_dir);

    path.push("../inputs.json");
    if !path.exists() {
        path = PathBuf::from("inputs.json");
    }

    let config_str = fs::read_to_string(&path).expect("Failed to read inputs.json");
    serde_json::from_str(&config_str).expect("Failed to parse inputs.json")
}

fn json_to_tokens(v: &serde_json::Value) -> proc_macro2::TokenStream {
    match v {
        serde_json::Value::Null => quote! { () },
        serde_json::Value::Bool(b) => {
            if *b {
                quote! { true }
            } else {
                quote! { false }
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                let lit = proc_macro2::Literal::i64_unsuffixed(i);
                quote! { #lit }
            } else if let Some(f) = n.as_f64() {
                let lit = proc_macro2::Literal::f64_unsuffixed(f);
                quote! { #lit }
            } else {
                quote! { 0 }
            }
        }
        serde_json::Value::String(s) => quote! { #s },
        serde_json::Value::Array(arr) => {
            let elems = arr.iter().map(json_to_tokens);
            quote! { [ #(#elems),* ] }
        }
        serde_json::Value::Object(_) => {
            panic!("Nested objects are not supported in json_to_tokens");
        }
    }
}

#[proc_macro_attribute]
pub fn benchmark_input(args: TokenStream, input: TokenStream) -> TokenStream {
    let test_name_lit = parse_macro_input!(args as LitStr);
    let test_name = test_name_lit.value();

    let ast = parse_macro_input!(input as DeriveInput);
    let struct_ident = &ast.ident;
    let struct_name = struct_ident.to_string();

    let config = load_config();
    let case = config
        .cases
        .iter()
        .find(|c| c.test == test_name)
        .unwrap_or_else(|| panic!("No case found in inputs.json for test '{}'", test_name));

    let mut generated_instances = Vec::new();

    for input_val in &case.inputs {
        let obj = input_val
            .as_object()
            .expect("Benchmark inputs must be JSON objects");

        let mut field_assigns = Vec::new();
        for (k, v) in obj {
            let field_ident = format_ident!("{}", k);
            let val_tokens = json_to_tokens(v);
            field_assigns.push(quote! {
                #field_ident: #val_tokens
            });
        }

        generated_instances.push(quote! {
            #struct_ident {
                #(#field_assigns),*
            }
        });
    }

    let static_ident = format_ident!("{}_INPUTS", struct_name.to_uppercase());

    let num_instances = generated_instances.len();

    let expanded = quote! {
        #ast

        #[allow(clippy::approx_constant)]
        pub static #static_ident: &[#struct_ident; #num_instances] = &[
            #(#generated_instances),*
        ];
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn generate_benchmarks(_input: TokenStream) -> TokenStream {
    let config = load_config();

    let mut runner_statements = Vec::new();

    for case in config.cases {
        let test_name_ident = format_ident!("{}", case.test);
        let struct_name = format!("{}Input", case.test);
        let static_ident = format_ident!("{}_INPUTS", struct_name.to_uppercase());
        let repetitions = case.repetitions;

        for lib in case.libraries {
            let lib_ident = format_ident!("{}", lib);
            let stmt = quote! {
                crate::run_and_log(
                    platform,
                    &crate::suites::#lib_ident::#test_name_ident,
                    crate::inputs::#static_ident,
                    #repetitions
                );
            };

            if let Some(platforms) = &case.platforms {
                let platform_checks = platforms.iter().map(|p| quote! { platform.id() == #p });
                runner_statements.push(quote! {
                    if #(#platform_checks)||* {
                        #stmt
                    }
                });
            } else {
                runner_statements.push(stmt);
            }
        }
    }

    let expanded = quote! {
        pub fn run_all_benchmarks<P: crate::BenchmarkPlatform>(platform: &mut P) {
            #(#runner_statements)*
        }
    };

    TokenStream::from(expanded)
}

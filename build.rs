//! SX128x Radio Driver
//! Copyright 2018 Ryan Kurte


extern crate bindgen;

use std::env;
use std::boxed::Box;
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;
use std::error::Error;
use std::collections::HashSet;

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn main() {
    let out_path = PathBuf::new(); //from(env::var("OUT_DIR").unwrap());
    let src_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
        ]
        .into_iter()
        .collect(),
    );

    // Build bindings
    println!("Generating bindings");
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .parse_callbacks(Box::new(ignored_macros))
        .use_core()
        .ctypes_prefix("libc")
        .header("src/wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    // Open a file for writing
    let binding_path = src_path.join("src/sx1280/mod.rs");
    let mut file = match File::create(&binding_path) {
        Err(e) => panic!("Error opening file {}: {}", binding_path.display(), e.description()),
        Ok(f) => f,
    };

    // Patch
    file.write_all(b"#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]\nuse libc;\n\n").unwrap();

    // Write bindings
    bindings
        .write(Box::new(file))
        .expect("Couldn't write bindings!");

    // Build libraries
    println!("Building library");
    cc::Build::new()
        .file("src/sx1280/sx1280.c")
        .include("src/sx1280")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-int-conversion")
        .flag("-Wno-implicit-function-declaration")
        .compile(out_path.join("sx1280").to_str().unwrap());

    // Link the library
    println!("cargo:rustc-link-lib=sx1280");
}

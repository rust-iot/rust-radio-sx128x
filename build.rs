//! SX128x Radio Driver
//! Copyright 2018 Ryan Kurte


extern crate bindgen;
extern crate git2;
use git2::Repository;

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
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Clone library
    let repo_url = "https://github.com/ryankurte/libsx128x";

    let repo_path = match env::var("LIBSX128x_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => {
            let mut repo_path = PathBuf::new();
            repo_path.push("libsx128x");
            repo_path
        }
    };

    let _repo = match repo_path.exists() {
        false => {
            println!("Cloning: '{}' into: '{:?}'", repo_url, repo_path);
            match Repository::clone(repo_url, repo_path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            }
        },
        true => {
            println!("Connecting to repo: '{:?}'", repo_path);
            match Repository::open(repo_path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            }
        }
    };

    // Build bindings
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

    println!("Generating bindings");
    let bindings = bindgen::Builder::default()
        .generate_comments(false)
        .parse_callbacks(Box::new(ignored_macros))
        .use_core()
        .ctypes_prefix("libc")
        //.clang_arg("-I/usr/include")
        .clang_arg("-Ilibsx128x/lib")
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    // Open a file for writing
    let binding_path = src_path.join("src/bindings.rs");
    let mut file = match File::create(&binding_path) {
        Err(e) => panic!("Error opening file {}: {}", binding_path.display(), e.description()),
        Ok(f) => f,
    };

    // Patch generated output to suppress warnings
    file.write_all(b"#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]\nuse libc;\n\n").unwrap();

    // Write bindings
    bindings
        .write(Box::new(file))
        .expect("Couldn't write bindings!");

    // Build libraries
    println!("Building library");
    cc::Build::new()
        .file("libsx128x/lib/sx1280.c")
        .file("libsx128x/lib/sx1280-hal.c")
        .include("libsx128x/lib")
        .debug(true)
        .flag("-Wno-unused-parameter")
        .flag("-Wno-int-conversion")
        .flag("-Wno-implicit-function-declaration")
        .compile("sx1280");

    // Link the library
    println!("cargo:rustc-link-lib=sx1280");
}

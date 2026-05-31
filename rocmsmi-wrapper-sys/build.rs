// Copyright 2026 Sustain AI
// Licensed under the Apache License, Version 2.0 (the "License");
// SPDX-License-Identifier: Apache-2.0

fn main() {
    println!("cargo:rerun-if-changed=input/allow_list.txt");
    let allowlist =
        std::fs::read_to_string("input/allow_list.txt").expect("Failed to read allow_list.txt");
    let rocm_include = std::env::var("ROCM_INCLUDE_PATH")
        .unwrap_or_else(|_| "/opt/rocm-7.2.3/include".to_string());
    let header_path = format!("{}/rocm_smi/rocm_smi.h", rocm_include);
    let mut builder = bindgen::Builder::default()
        .clang_arg("-xc++")
        .clang_arg(format!("-I{}", rocm_include))
        .header(&header_path)
        .dynamic_library_name("librocm_smi")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        });
    for line in allowlist.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            builder = builder.allowlist_item(line);
        }
    }
    let bindings = builder
        .generate_comments(false)
        .generate()
        .expect("Unable to generate bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("latest.rs"))
        .expect("Unable to write bindings");
}

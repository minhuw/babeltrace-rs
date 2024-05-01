use anyhow::{anyhow, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build_babeltrace(out_path: &Path) -> Result<PathBuf> {
    let src_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("babeltrace");
    let work_path: PathBuf = out_path.join("build/");

    std::fs::create_dir_all(out_path).unwrap();
    std::fs::create_dir_all(&work_path).unwrap();

    // cp is very likely to be available on any system with autotools
    let _ = Command::new("cp")
        .arg("-r")
        .arg(&src_path)
        .arg(&work_path)
        .output()
        .expect("unable to copy babeltrace");

    let Ok(compiler) = cc::Build::new().try_get_compiler() else {
        panic!("a C compiler is required to compile babeltrace")
    };

    println!(
        "cargo:warning=cc: {}, cflags: {}, debug: {}, work_path: {}",
        compiler.path().display(),
        compiler.cflags_env().to_string_lossy(),
        std::env::var("PROFILE").unwrap(),
        work_path.display()
    );

    // babeltrace could not be compiled with -Wall and -Wextra
    let cflags = compiler
        .cflags_env()
        .to_string_lossy()
        .replace("-Wall", "")
        .replace("-Wextra", "");

    let extra_flags = match std::env::var("PROFILE").unwrap().as_str() {
        "debug" => "--disable-Werror",
        _ => "",
    };
    let configure_flags = format!(
        "--disable-python-bindings --disable-python-plugins --disable-man-pages {extra_flags}"
    );

    let compile_result = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "./bootstrap && ./configure --prefix {} {} && make -j && make install",
            out_path.to_string_lossy(),
            configure_flags
        ))
        .current_dir(work_path.join("babeltrace"))
        .env("CC", compiler.path())
        .env("CFLAGS", cflags)
        .env("BABELTRACE_DEV_MODE", "1")
        .env("BABELTRACE_MINIMAL_LOG_LEVEL", "TRACE")
        .output()
        .expect("unable to compile babeltrace");

    if compile_result.status.success() {
        Ok(out_path.to_path_buf())
    } else {
        eprintln!(
            "unable to compile babeltrace\n stdout: {}, stderr: {}",
            String::from_utf8_lossy(&compile_result.stdout),
            String::from_utf8_lossy(&compile_result.stderr)
        );
        Err(anyhow!("unable to compile babeltrace"))
    }
}

fn main() -> anyhow::Result<()> {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let babeltrace = build_babeltrace(&out_path)?;
    let lib_path = babeltrace.join("lib/");
    let include_path = babeltrace.join("include/");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    println!("cargo::rustc-link-search=native={}", lib_path.display());
    println!("cargo::rustc-link-lib=dylib=babeltrace2");
    println!("cargo::rustc-link-lib=glib-2.0");
    println!("cargo::rustc-link-lib=gmodule-2.0");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .wrap_static_fns(true)
        .clang_arg(format!("-I{}", include_path.display()))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("couldn't write bindings!");

    println!("cargo:warning=babeltrace={}", babeltrace.display());
    println!("cargo:include={}", include_path.display());
    println!("cargo:lib={}", lib_path.display());

    Ok(())
}

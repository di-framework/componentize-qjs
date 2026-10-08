use std::env;
use std::path::PathBuf;
use std::process::Command;

// `vendor/libzwasm.a` is zwasm b84f9f97 built for wasm32-wasi with
// `-Dengine=interp` (the archive already tested by wasi-wit-zwasm). Zig has
// no wasip2 target, so the archive stays preview 1. This component is wasip2.
// Do not rebuild the archive here; the symbols this runtime calls are in it.
fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest.join("vendor");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=zwasm");
    println!("cargo:rerun-if-changed=missing.c");
    println!("cargo:rerun-if-changed=vendor/libzwasm.a");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("zwasm_missing.o");
    let status = Command::new(env::var("ZIG").unwrap_or_else(|_| "zig".into()))
        .args(["cc", "-target", "wasm32-wasi", "-c", "missing.c", "-o"])
        .arg(&out)
        .status()
        .expect("zig cc");
    if !status.success() {
        panic!("zig cc missing.c failed");
    }
    println!("cargo:rustc-link-arg={}", out.display());
}

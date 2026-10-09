use std::env;
use std::path::PathBuf;

// `vendor/libzwasm.a` is zwasm b84f9f97 built for wasm32-wasi with
// `-Dengine=interp` (the archive already tested by wasi-wit-zwasm). Zig has
// no wasip2 target, so the archive stays preview 1. This component is wasip2.
// Do not rebuild the archive here; the symbols this runtime calls are in it.
fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest.join("vendor");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=zwasm");
    println!("cargo:rerun-if-changed=vendor/libzwasm.a");
}

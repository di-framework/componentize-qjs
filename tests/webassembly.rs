//! Guest WebAssembly.instantiate backed by zwasm.
mod common;

use std::fs;

use wasmtime::component::Val;

use common::{ComponentInstance, componentize_qjs, run_cli_build};

const HELLO: &str = "new Uint8Array([0,97,115,109,1,0,0,0,1,5,1,96,0,1,127,3,2,1,0,7,8,1,4,109,97,105,110,0,0,10,6,1,4,0,65,42,11])";

/// `(import "env" "add" (func (param i32 i32) (result i32)))` then `main` = add(20, 22).
const ADD: &str = "new Uint8Array([0,97,115,109,1,0,0,0,1,11,2,96,2,127,127,1,127,96,0,1,127,2,11,1,3,101,110,118,3,97,100,100,0,0,3,2,1,1,7,8,1,4,109,97,105,110,0,1,10,10,1,8,0,65,20,65,22,16,0,11,0,13,4,110,97,109,101,1,6,1,0,3,97,100,100])";

/// One page of memory, exported as `mem`, and `grow` which grows one page.
const MEM: &str = "new Uint8Array([0,97,115,109,1,0,0,0,1,5,1,96,0,1,127,3,2,1,0,5,3,1,0,1,7,14,2,3,109,101,109,2,0,4,103,114,111,119,0,0,10,8,1,6,0,65,1,64,0,11])";

/// `(func (export "id") (param i32) (result i32) local.get 0)`.
const ID_I32: &str = "new Uint8Array([0,97,115,109,1,0,0,0,1,6,1,96,1,127,1,127,3,2,1,0,7,6,1,2,105,100,0,0,10,6,1,4,0,32,0,11])";

/// `(func (export "id") (param i64) (result i64) local.get 0)`.
const ID_I64: &str = "new Uint8Array([0,97,115,109,1,0,0,0,1,6,1,96,1,126,1,126,3,2,1,0,7,6,1,2,105,100,0,0,10,6,1,4,0,32,0,11])";

/// One memory exported as both `a` and `b`, plus `grow` of one page.
const MEM_ALIAS: &str = "new Uint8Array([0,97,115,109,1,0,0,0,1,5,1,96,0,1,127,3,2,1,0,5,3,1,0,1,7,16,3,1,97,2,0,1,98,2,0,4,103,114,111,119,0,0,10,8,1,6,0,65,1,64,0,11])";

fn wit_answer() -> &'static str {
    "package test:wasm;\nworld wasm { export answer: func() -> u32; }\n"
}

fn run_js(js: &str) -> u32 {
    let (output, _dir) = run_cli_build(wit_answer(), js, &[]);
    let wasm = fs::read(&output).unwrap();
    let mut inst = ComponentInstance::from_wasm(wasm, vec![], vec![]).unwrap();
    match inst.call1("answer", &[]) {
        Val::U32(value) => value,
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn guest_without_wasm_import_still_runs() {
    let (output, _dir) = run_cli_build(wit_answer(), "export function answer() { return 7; }", &[]);
    let wasm = fs::read(&output).unwrap();
    let mut inst = ComponentInstance::from_wasm(wasm, vec![], vec![]).unwrap();
    assert_eq!(inst.call1("answer", &[]), Val::U32(7));
}

#[test]
fn instantiate_exported_main_returns_42() {
    let js = format!(
        "const bytes = {HELLO};
         export function answer() {{
           const {{ instance }} = WebAssembly.instantiate(bytes, {{}});
           return instance.exports.main();
         }}"
    );
    assert_eq!(run_js(&js), 42);
}

#[test]
fn instantiate_calls_host_function_import() {
    let js = format!(
        "const bytes = {ADD};
         export function answer() {{
           const {{ instance }} = WebAssembly.instantiate(bytes, {{
             env: {{ add(a, b) {{ return a + b; }} }}
           }});
           return instance.exports.main();
         }}"
    );
    assert_eq!(run_js(&js), 42);
}

#[test]
fn missing_or_mistyped_import_is_catchable() {
    let js = format!(
        "const bytes = {ADD};
         export function answer() {{
           let caught = 0;
           try {{ WebAssembly.instantiate(bytes, {{}}); }} catch (e) {{ caught += 1; }}
           try {{ WebAssembly.instantiate(bytes, {{ env: {{ add: 1 }} }}); }} catch (e) {{ caught += 1; }}
           return caught;
         }}"
    );
    assert_eq!(run_js(&js), 2);
}

#[test]
fn repeated_instantiate_reuses_host_import_slots() {
    let js = format!(
        "const bytes = {ADD};
         export function answer() {{
           for (let i = 0; i < 40; i++) {{
             const {{ instance }} = WebAssembly.instantiate(bytes, {{
               env: {{ add(a, b) {{ return a + b; }} }}
             }});
             if (instance.exports.main() !== 42) return 1;
           }}
           return 0;
         }}"
    );
    assert_eq!(run_js(&js), 0);
}

#[test]
fn i32_arguments_wrap_and_i64_is_bigint() {
    let js = format!(
        "const i32bytes = {ID_I32};
         const i64bytes = {ID_I64};
         export function answer() {{
           const i32 = WebAssembly.instantiate(i32bytes, {{}}).instance.exports.id;
           if (i32(4294967295) !== -1) return 1;
           if (i32(2147483648) !== -2147483648) return 2;
           if (i32(NaN) !== 0) return 3;
           const i64 = WebAssembly.instantiate(i64bytes, {{}}).instance.exports.id;
           if (i64(1n) !== 1n) return 4;
           if (i64(9223372036854775808n) !== -9223372036854775808n) return 5;
           return 0;
         }}"
    );
    assert_eq!(run_js(&js), 0);
}

#[test]
fn aliased_memory_export_detaches_both_buffers() {
    let js = format!(
        "const bytes = {MEM_ALIAS};
         export function answer() {{
           const {{ instance }} = WebAssembly.instantiate(bytes, {{}});
           const first = instance.exports.a.buffer;
           const second = instance.exports.b.buffer;
           new Uint8Array(first)[0] = 0x5a;
           instance.exports.grow();
           if (first.byteLength !== 0) return 1;
           if (second.byteLength !== 0) return 2;
           const view = new Uint8Array(instance.exports.b.buffer);
           if (view.byteLength !== 131072) return 3;
           if (view[0] !== 0x5a) return 4;
           return 0;
         }}"
    );
    assert_eq!(run_js(&js), 0);
}

#[test]
fn memory_grow_detaches_the_old_buffer() {
    let js = format!(
        "const bytes = {MEM};
         export function answer() {{
           const {{ instance }} = WebAssembly.instantiate(bytes, {{}});
           const mem = instance.exports.mem;
           const old = mem.buffer;
           const view = new Uint8Array(old);
           view[0] = 0x5a;
           const prev = instance.exports.grow();
           if (prev !== 1) return 1;
           if (old.byteLength !== 0) return 2;
           if (view.buffer.byteLength !== 0) return 3;
           const next = new Uint8Array(mem.buffer);
           if (next.buffer.byteLength !== 131072) return 4;
           if (next[0] !== 0x5a) return 5;
           if (next[65536] !== 0) return 6;
           return 0;
         }}"
    );
    assert_eq!(run_js(&js), 0);
}

#[test]
fn vendored_wasm_pqc_subtle_init_returns_callable_export() {
    let dir = tempfile::TempDir::new().unwrap();
    let src = dir.path().join("src");
    let pkg = dir.path().join("node_modules").join("wasm-pqc-subtle");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&pkg).unwrap();
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wasm-pqc-subtle");
    fs::copy(fixture.join("package.json"), pkg.join("package.json")).unwrap();
    fs::copy(fixture.join("index.js"), pkg.join("index.js")).unwrap();
    fs::write(
        src.join("main.js"),
        r#"import init from "wasm-pqc-subtle";
           export function answer() {
             const exports = init();
             return exports.main();
           }"#,
    )
    .unwrap();
    let wit = dir.path().join("test.wit");
    fs::write(&wit, wit_answer()).unwrap();
    let output = dir.path().join("output.wasm");
    componentize_qjs()
        .arg("--wit")
        .arg(&wit)
        .arg("--js")
        .arg(src.join("main.js"))
        .arg("--module-root")
        .arg(dir.path())
        .arg("--output")
        .arg(&output)
        .assert()
        .success();
    let wasm = fs::read(&output).unwrap();
    let mut inst = ComponentInstance::from_wasm(wasm, vec![], vec![]).unwrap();
    assert_eq!(inst.call1("answer", &[]), Val::U32(42));
}

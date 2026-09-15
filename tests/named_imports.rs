use componentize_qjs::{ComponentizeOpts, Runtime, componentize};
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker};

#[tokio::test]
async fn two_imports_of_one_interface_are_independently_callable() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let wit_path = directory.path().join("named.wit");
    std::fs::write(&wit_path, r#"
        package test:named;
        interface query { value: func() -> u32; }
        world application {
            import first: query;
            import second: query;
            export run: func() -> u32;
        }
    "#)?;
    for stub_wasi in [false, true] {
        let wasm = componentize(&ComponentizeOpts {
            wit_path: &wit_path,
            js_source: "import { value as first } from 'first'; import { value as second } from 'second'; export function run() { return first() * 100 + second(); }",
            js_path: None, module_root: None, world_name: Some("application"),
            stub_wasi, disable_gc: false, runtime: Runtime::DefaultSync,
        }).await?;
        let mut config = Config::new();
        config.wasm_component_model(true).wasm_component_model_implements(true);
        let engine = Engine::new(&config)?;
        let component = Component::new(&engine, &wasm)?;
        let mut linker = Linker::<()>::new(&engine);
        linker.allow_shadowing(true);
        linker.define_unknown_imports_as_traps(&component)?;
        linker.instance("first")?.func_wrap("value", |_, (): ()| Ok((17u32,)))?;
        linker.instance("second")?.func_wrap("value", |_, (): ()| Ok((29u32,)))?;
        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &component)?;
        let run = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
        assert_eq!(run.call(&mut store, ())?.0, 1729);
    }
    Ok(())
}

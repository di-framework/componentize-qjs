//! WASI import stubbing for snapshotted components.
//!
//! The approach:
//! 1. Decode the snapshotted component to extract its WIT world
//! 2. Create a "stub world" where the WASI imports become exports
//! 3. Use `dummy_module` to generate a core module with trap implementations
//! 4. Encode it as a stub component
//! 5. Use `wasm-compose` to compose the stub into the original component

use anyhow::{Context, Result, bail};
use indexmap::IndexMap;
use wasm_compose::{composer::ComponentComposer, config::Config as ComposeConfig};
use wit_component::{ComponentEncoder, StringEncoding, dummy_module, embed_component_metadata};
use wit_parser::decoding::{DecodedWasm, decode};
use wit_parser::{Docs, ManglingAndAbi, Resolve, Stability, World, WorldItem, WorldKey};

/// Stub all WASI imports in a component, producing a self-contained component.
pub fn stub_wasi_imports(component: &[u8]) -> Result<Vec<u8>> {
    stub_imports(component, |name| name.starts_with("wasi:"))
}

/// Stub componentization-only imports that must not leak into final components.
pub fn stub_internal_imports(component: &[u8]) -> Result<Vec<u8>> {
    stub_imports(component, |name| name == "local:init/module-loader")
}

fn stub_imports(component: &[u8], should_stub: impl Fn(&str) -> bool) -> Result<Vec<u8>> {
    let decoded = decode(component).context("failed to decode component WIT")?;
    let (resolve, world_id) = match decoded {
        DecodedWasm::Component(resolve, world_id) => (resolve, world_id),
        _ => bail!("expected a component, got a WIT package"),
    };

    let world = &resolve.worlds[world_id];

    let imports: IndexMap<WorldKey, WorldItem> = world
        .imports
        .clone()
        .into_iter()
        .filter(|(key, _)| should_stub(&resolve.name_world_key(key)))
        .collect();

    if imports.is_empty() {
        return Ok(component.to_vec());
    }

    let stub_component =
        make_stub_component(&resolve, world, &imports).context("failed to build stub component")?;

    let dir = tempfile::tempdir().context("failed to create composition directory")?;
    let component_path = dir.path().join("original.wasm");
    let stub_path = dir.path().join("stubs.wasm");

    std::fs::write(&component_path, component).context("failed to stage original component")?;
    std::fs::write(&stub_path, stub_component).context("failed to stage stub component")?;

    let config = ComposeConfig {
        dir: dir.path().to_path_buf(),
        definitions: vec!["stubs.wasm".into()],
        // wasm-compose uses default features for output validation. Validate below
        // with the same proposal support as the component encoder instead.
        skip_validation: true,
        ..Default::default()
    };

    let composed = ComponentComposer::new(&component_path, &config)
        .compose()
        .context("failed to compose stub component")?;
    let composed = preserve_import_annotations(component, &composed)?;
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&composed)
        .context("failed to validate composed component")?;
    Ok(composed)
}

/// Build a component that exports trap implementations for the given imports.
fn make_stub_component(
    resolve: &Resolve,
    original_world: &World,
    imports: &IndexMap<WorldKey, WorldItem>,
) -> Result<Vec<u8>> {
    let mut stub_resolve = resolve.clone();
    let stub_world_id = stub_resolve.worlds.alloc(World {
        name: "wasi-stubs".to_string(),
        imports: IndexMap::new(),
        exports: imports.clone(),
        package: original_world.package,
        docs: Docs::default(),
        stability: Stability::default(),
        includes: Vec::new(),
        span: Default::default(),
    });

    let mut core_module = dummy_module(&stub_resolve, stub_world_id, ManglingAndAbi::Standard32);

    embed_component_metadata(
        &mut core_module,
        &stub_resolve,
        stub_world_id,
        StringEncoding::UTF8,
    )
    .context("failed to embed component metadata in stub module")?;

    ComponentEncoder::default()
        .module(&core_module)
        .unwrap()
        .validate(true)
        .encode()
        .context("failed to encode stub component")
}

/// Compose provider components while retaining named host-interface identity.
pub fn compose_with_definitions(component_path: &std::path::Path, definitions: Vec<std::path::PathBuf>) -> Result<Vec<u8>> {
    let original = std::fs::read(component_path)?;
    let config = ComposeConfig { definitions, skip_validation: true, ..Default::default() };
    let composed = ComponentComposer::new(component_path, &config).compose()?;
    let composed = preserve_import_annotations(&original, &composed)?;
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all()).validate_all(&composed)?;
    Ok(composed)
}

/// wasm-compose 0.258 retains import types and labels but drops extern-name
/// annotations on the outer component. Copy those annotations from the input;
/// indices, types, nested components and all other sections remain unchanged.
fn preserve_import_annotations(original: &[u8], composed: &[u8]) -> Result<Vec<u8>> {
    use wasm_encoder::reencode::{ReencodeComponent, RoundtripReencoder};
    use wasmparser::{Parser, Payload};
    let mut names = std::collections::HashMap::new();
    let mut depth = 0;
    for payload in Parser::new(0).parse_all(original) {
        match payload? {
            Payload::Version { .. } => depth += 1,
            Payload::End(_) => depth -= 1,
            Payload::ComponentImportSection(section) if depth == 1 => {
                for import in section {
                    let name = import?.name;
                    names.insert(name.name, name);
                }
            }
            _ => {}
        }
    }
    let mut output = wasm_encoder::Component::new();
    let mut depth = 0;
    for payload in Parser::new(0).parse_all(composed) {
        let payload = payload?;
        match payload {
            Payload::Version { .. } => depth += 1,
            Payload::End(_) => depth -= 1,
            Payload::ComponentImportSection(section) if depth == 1 => {
                let mut imports = wasm_encoder::ComponentImportSection::new();
                for import in section {
                    let import = import?;
                    let name = names.get(import.name.name).copied().unwrap_or(import.name);
                    imports.import(name, RoundtripReencoder.component_type_ref(import.ty)?);
                }
                output.section(&imports);
            }
            _ if depth == 1 => {
                if let Some((id, range)) = payload.as_section() {
                    output.section(&wasm_encoder::RawSection { id, data: &composed[usize::try_from(range.start)?..usize::try_from(range.end)?] });
                }
            }
            _ => {}
        }
    }
    Ok(output.finish())
}

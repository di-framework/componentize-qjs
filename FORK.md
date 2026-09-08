# Fork notes

This is a temporary fork of
[`andreiltd/componentize-qjs`](https://github.com/andreiltd/componentize-qjs)
`v0.4.4` (Apache-2.0).

## Why it exists

Stock componentize-qjs 0.4.4 / jco 1.32.1 uses wasmtime **47**. During Wizer,
unknown imports are stubbed with sync `func_new`. Guest worlds that import
`async func` (wasmCloud postgres, keyvalue, blobstore, messaging, secrets,
outgoing HTTP) fail with `type mismatch with async`.

Wasmtime **48** can stub those imports with `func_new_concurrent` when
`Config::concurrency_support(true)` is set.

## What changed

- `wasmtime` / `wasmtime-wasi` / `wasmtime-wizer` / preview1 adapter: 47 → 48
- `config.concurrency_support(true)`
- `define_unknown_imports_as_traps` for unknown imports (including guest
  `async func`s)
- Overlay only the WASI P2 interfaces the 0.4.4 prebuilt runtime actually
  calls during init (`random`, `wall-clock`, `cli` environment/exit, and a
  resource-free `monotonic-clock` `now`/`resolution`). Full P2/P3 `add_to_linker`
  disagrees with the prebuilt `wasi:io@0.2.12` `error` resource.

The embedded QuickJS **runtime.wasm** is still the 0.4.4 prebuilt, vendored
under `crates/core/prebuilt/` (gitignored `*.wasm` has an exception for that
directory). This fork does not rebuild `wasm32-wasip2` artifacts. CI uses those
files instead of compiling the runtime.

## npm

`@di-framework/componentize-qjs` is a JS wrapper. The native CLI is in
optional platform packages. GitHub Actions publishes them from
`.github/workflows/release.yml` using npm OIDC trusted publishing (no token).
The wrapper (`0.4.4-di.2`) publishes to the `di` dist-tag. Each OS package
publishes one version per CPU (`0.4.4-di.2-arm64`, `0.4.4-di.2-x64`) to the
`di-arm64` / `di-x64` dist-tags. npm cannot install two versions of the same
package as optional dependencies, so the wrapper aliases those versions
(`npm:@di-framework/componentize-qjs-darwin@0.4.4-di.2-arm64`) and relies on
each tarball's `os` / `cpu` fields to skip the wrong machine.
On npmjs.com, the trusted publisher must be:

- Organization: `di-framework`
- Repository: `componentize-qjs`
- Workflow filename: `release.yml`
- Environment name: empty
- Allow npm publish: checked

Create these three packages (not one per CPU):

| Package | Versions |
| --- | --- |
| `@di-framework/componentize-qjs-darwin` | `0.4.4-di.2-arm64`, `0.4.4-di.2-x64` |
| `@di-framework/componentize-qjs-linux` | `0.4.4-di.2-x64`, `0.4.4-di.2-arm64` |
| `@di-framework/componentize-qjs-win32` | `0.4.4-di.2-x64` |

There is no postinstall download. `npm install` / `bun install` selects the
matching optional dependency.

```js
const { nativeCliPath } = require('@di-framework/componentize-qjs');
nativeCliPath(); // absolute path, or undefined if the platform package is missing
```

## Upstream

Please prefer a patch on `andreiltd/componentize-qjs` once they bump wasmtime.
This fork should then be retired.

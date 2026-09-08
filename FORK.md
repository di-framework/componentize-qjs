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

Everything publishes as `@di-framework/componentize-qjs` (one npm package, one
trusted publisher). GitHub Actions publishes from `.github/workflows/release.yml`
using npm OIDC (no token).

| Version | Dist-tag | Contents |
| --- | --- | --- |
| `0.4.4-di.2` | `di` | JS wrapper |
| `0.4.4-di.2-darwin-arm64` | `di-darwin-arm64` | native CLI |
| `0.4.4-di.2-darwin-x64` | `di-darwin-x64` | native CLI |
| `0.4.4-di.2-linux-x64` | `di-linux-x64` | native CLI |
| `0.4.4-di.2-linux-arm64` | `di-linux-arm64` | native CLI |
| `0.4.4-di.2-win32-x64` | `di-win32-x64` | native CLI |

npm cannot install two versions of the same package as optional dependencies, so
the wrapper aliases each native version into an unscoped folder
(`componentize-qjs-darwin-arm64`: `npm:@di-framework/componentize-qjs@0.4.4-di.2-darwin-arm64`)
and each tarball's `os` / `cpu` fields skip the wrong machine.

On npmjs.com, the trusted publisher for `@di-framework/componentize-qjs` is:

- Organization: `di-framework`
- Repository: `componentize-qjs`
- Workflow filename: `release.yml`
- Environment name: empty
- Allow npm publish: checked

There is no postinstall download and no extra platform packages. `npm install` /
`bun install` of the wrapper selects the matching optional version.

```js
const { nativeCliPath } = require('@di-framework/componentize-qjs');
nativeCliPath(); // absolute path, or undefined if the platform package is missing
```

## Upstream

Please prefer a patch on `andreiltd/componentize-qjs` once they bump wasmtime.
This fork should then be retired.

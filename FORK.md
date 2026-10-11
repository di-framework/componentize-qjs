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

The embedded QuickJS runtimes are vendored under `crates/core/prebuilt/`
(gitignored `*.wasm` has an exception for that directory), and CI embeds those
files instead of compiling the runtime. Since di.4, the default
**runtime.wasm** is rebuilt from this fork's `crates/runtime` source (see
[64-bit integers are BigInt](#64-bit-integers-are-bigint-di5)). The other three
(`runtime-sync.wasm`, `runtime-opt-size.wasm`, `runtime-opt-size-sync.wasm`) are
still the upstream 0.4.4 prebuilts.

## npm

Everything publishes as `@di-framework/componentize-qjs` (one npm package, one
trusted publisher). GitHub Actions publishes from `.github/workflows/release.yml`
using npm OIDC (no token).

| Version | Dist-tag | Contents |
| --- | --- | --- |
| `0.4.4-di.5` | `di` | JS wrapper |
| `0.4.4-di.5-darwin-arm64` | `di-darwin-arm64` | native CLI |
| `0.4.4-di.5-darwin-x64` | `di-darwin-x64` | native CLI |
| `0.4.4-di.5-linux-x64` | `di-linux-x64` | native CLI |
| `0.4.4-di.5-linux-arm64` | `di-linux-arm64` | native CLI |
| `0.4.4-di.5-win32-x64` | `di-win32-x64` | native CLI |

npm cannot install two versions of the same package as optional dependencies, so
the wrapper aliases each native version into an unscoped folder
(`componentize-qjs-darwin-arm64`: `npm:@di-framework/componentize-qjs@0.4.4-di.5-darwin-arm64`)
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

## Named imports (di.3)

Enable `cm-implements` in the snapshot engine. Composition output is validated
with proposal support matching the WIT encoder, replacing wasm-compose’s
default-feature validation. The regression test invokes two imports of the same
interface with distinct host implementations, both with and without WASI stubbing.

Composition also preserves extern-name annotations on outer imports. wasm-compose
0.258 otherwise keeps labels but loses `(implements ...)`, breaking wasmCloud
routing. `componentize-qjs compose <input> --definition <provider> -o <output>`
uses the same validated path for subsequent provider composition (including SQLite).

## 64-bit integers are BigInt (di.5)

WIT `s64` and `u64` values reach JavaScript as `BigInt`, as in jco. Export
parameters and import results above 2^53 used to arrive as a rounded `Number`
(di-framework/platform#153). Going the other way, the runtime accepts a `BigInt`
(wrapped modulo 2^64, like `BigInt.asIntN` / `BigInt.asUintN`) or a `Number`
that is an integer within `Number.MAX_SAFE_INTEGER`. Any other `Number` traps
instead of storing a rounded value, and `u64` rejects negative Numbers.

Guest code that does arithmetic on a 64-bit import result with Number operands
(`n + 1`), or passes one to `JSON.stringify`, must now convert explicitly.

Only the default runtime (`crates/core/prebuilt/runtime.wasm`) is rebuilt, with
`COMPONENTIZE_QJS_REBUILD_RUNTIME=1 cargo build -p componentize-qjs --release`,
then copied from `OUT_DIR` with its `.sha256` updated. The di.5 build used
rustup rustc 1.97.1 with the `wasm32-wasip2` target installed, plus wasi-sdk 33
and binaryen 130 (both pinned in `crates/core/build.rs`). The output is
reproducible byte for byte with that toolchain. A `cargo` that is not from
rustup and lacks the `wasm32-wasip2` std fails inside the nested build with
E0463, so put `~/.cargo/bin` first on `PATH`.

Because it is a `--release` build, `build.rs` runs
`wasm-opt -O3 --strip-debug --strip-producers`, so the prebuilt has no `name`
or `producers` section, like the upstream prebuilts. Panic messages (such as
`expected s64: ...`) still reach guest stderr, but wasmtime trap backtraces show
`wasm function N`. For symbolized frames, rebuild without `--release`
(`COMPONENTIZE_QJS_REBUILD_RUNTIME=1 cargo build -p componentize-qjs`) and use
that runtime locally.

The `--sync` and `--opt-size` variants are still the upstream 0.4.4 prebuilts
and keep the old Number behaviour. So does a binary built with
`--no-default-features` (its default runtime aliases `runtime-sync.wasm`) and
the Cargo / napi `opt-size` feature.

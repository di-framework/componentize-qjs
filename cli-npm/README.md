# @di-framework/componentize-qjs

Fork of [`componentize-qjs`](https://github.com/andreiltd/componentize-qjs) `v0.4.4`
with a wasmtime **48** Wizer host. Stock 0.4.4 cannot stub imported WIT
`async func`s (`type mismatch with async`). This CLI can.

The native binary is an optional platform package. There is no postinstall
download from GitHub releases.

```js
const { nativeCliPath } = require('@di-framework/componentize-qjs');
const bin = nativeCliPath();
```

```bash
npx @di-framework/componentize-qjs --wit ./wit --js ./guest.js -n application -o out.wasm
```

See [FORK.md](https://github.com/di-framework/componentize-qjs/blob/main/FORK.md).

#!/usr/bin/env node
'use strict';

const { spawn } = require('node:child_process');
const { nativeCliPath, platformPackageId } = require('../native-cli.cjs');

const bin = nativeCliPath();
if (bin === undefined) {
  process.stderr.write(
    `@di-framework/componentize-qjs: native CLI not installed (${platformPackageId()}).\n` +
      'Install the matching optional dependency, or set DI_FRAMEWORK_COMPONENTIZE_QJS.\n',
  );
  process.exit(1);
}

const child = spawn(bin, process.argv.slice(2), { stdio: 'inherit' });
child.on('error', (error) => {
  process.stderr.write(`${error}\n`);
  process.exit(1);
});
child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});

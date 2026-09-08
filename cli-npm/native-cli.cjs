'use strict';

const { existsSync } = require('node:fs');
const { createRequire } = require('node:module');
const { dirname, join } = require('node:path');

const PACKAGE_NAME = '@di-framework/componentize-qjs';
const requireFromHere = createRequire(__filename);

function platformPackageName(platform = process.platform, arch = process.arch) {
  return `${PACKAGE_NAME}-${platform}-${arch}`;
}

function nativeCliPath(platform = process.platform, arch = process.arch) {
  const name = platformPackageName(platform, arch);
  let packageJson;
  try {
    packageJson = requireFromHere.resolve(`${name}/package.json`);
  } catch {
    return undefined;
  }
  const bin = platform === 'win32' ? 'componentize-qjs.exe' : 'componentize-qjs';
  const resolved = join(dirname(packageJson), 'bin', bin);
  return existsSync(resolved) ? resolved : undefined;
}

module.exports = {
  PACKAGE_NAME,
  nativeCliPath,
  platformPackageName,
};

'use strict';

const { existsSync } = require('node:fs');
const { createRequire } = require('node:module');
const { dirname, join } = require('node:path');

const PACKAGE_NAME = '@di-framework/componentize-qjs';
const WRAPPER_VERSION = require('./package.json').version;
const requireFromHere = createRequire(__filename);

function platformAliasName(platform = process.platform, arch = process.arch) {
  return `componentize-qjs-${platform}-${arch}`;
}

function platformPackageVersion(
  platform = process.platform,
  arch = process.arch,
  wrapperVersion = WRAPPER_VERSION,
) {
  return `${wrapperVersion}-${platform}-${arch}`;
}

function platformPackageId(platform = process.platform, arch = process.arch) {
  return `${PACKAGE_NAME}@${platformPackageVersion(platform, arch)}`;
}

function packageMatchesPlatform(pkg, platform, arch) {
  if (Array.isArray(pkg.os) && pkg.os.length > 0 && !pkg.os.includes(platform)) {
    return false;
  }
  if (Array.isArray(pkg.cpu) && pkg.cpu.length > 0 && !pkg.cpu.includes(arch)) {
    return false;
  }
  if (
    typeof pkg.version === 'string' &&
    /-(?:darwin|linux|win32|android)-(?:arm64|x64)$/.test(pkg.version)
  ) {
    return pkg.version.endsWith(`-${platform}-${arch}`);
  }
  return true;
}

function nativeCliPath(platform = process.platform, arch = process.arch) {
  const names = [platformAliasName(platform, arch), PACKAGE_NAME];
  const bin = platform === 'win32' ? 'componentize-qjs.exe' : 'componentize-qjs';
  for (const name of names) {
    let packageJson;
    try {
      packageJson = requireFromHere.resolve(`${name}/package.json`);
    } catch {
      continue;
    }
    let pkg;
    try {
      pkg = requireFromHere(packageJson);
    } catch {
      pkg = {};
    }
    if (!packageMatchesPlatform(pkg, platform, arch)) continue;
    const resolved = join(dirname(packageJson), 'bin', bin);
    if (existsSync(resolved)) return resolved;
  }
  return undefined;
}

module.exports = {
  PACKAGE_NAME,
  WRAPPER_VERSION,
  nativeCliPath,
  platformAliasName,
  platformPackageVersion,
  platformPackageId,
};

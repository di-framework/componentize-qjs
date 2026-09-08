'use strict';

const { existsSync } = require('node:fs');
const { createRequire } = require('node:module');
const { dirname, join } = require('node:path');

const PACKAGE_NAME = '@di-framework/componentize-qjs';
const WRAPPER_VERSION = require('./package.json').version;
const requireFromHere = createRequire(__filename);

function platformPackageName(platform = process.platform) {
  return `${PACKAGE_NAME}-${platform}`;
}

function platformPackageVersion(arch = process.arch, wrapperVersion = WRAPPER_VERSION) {
  return `${wrapperVersion}-${arch}`;
}

function platformPackageId(platform = process.platform, arch = process.arch) {
  return `${platformPackageName(platform)}@${platformPackageVersion(arch)}`;
}

function packageMatchesArch(pkg, arch) {
  if (Array.isArray(pkg.cpu) && pkg.cpu.length > 0 && !pkg.cpu.includes(arch)) {
    return false;
  }
  if (typeof pkg.version === 'string' && /-(?:arm64|x64)$/.test(pkg.version)) {
    return pkg.version.endsWith(`-${arch}`);
  }
  return true;
}

function nativeCliPath(platform = process.platform, arch = process.arch) {
  const names = [platformPackageName(platform), `${PACKAGE_NAME}-${platform}-${arch}`];
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
    if (!packageMatchesArch(pkg, arch)) continue;
    const resolved = join(dirname(packageJson), 'bin', bin);
    if (existsSync(resolved)) return resolved;
  }
  return undefined;
}

module.exports = {
  PACKAGE_NAME,
  WRAPPER_VERSION,
  nativeCliPath,
  platformPackageName,
  platformPackageVersion,
  platformPackageId,
};

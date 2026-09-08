'use strict';

const assert = require('node:assert/strict');
const {
  PACKAGE_NAME,
  WRAPPER_VERSION,
  platformAliasName,
  platformPackageVersion,
  platformPackageId,
  nativeCliPath,
} = require('./native-cli.cjs');

assert.equal(PACKAGE_NAME, '@di-framework/componentize-qjs');
assert.equal(WRAPPER_VERSION, '0.4.4-di.2');
assert.equal(platformAliasName('darwin', 'arm64'), 'componentize-qjs-darwin-arm64');
assert.equal(platformPackageVersion('darwin', 'arm64'), '0.4.4-di.2-darwin-arm64');
assert.equal(platformPackageVersion('linux', 'x64'), '0.4.4-di.2-linux-x64');
assert.equal(
  platformPackageId('darwin', 'arm64'),
  '@di-framework/componentize-qjs@0.4.4-di.2-darwin-arm64',
);
assert.equal(
  platformPackageId('linux', 'x64'),
  '@di-framework/componentize-qjs@0.4.4-di.2-linux-x64',
);
assert.equal(nativeCliPath('plan9', 'x64'), undefined);
console.log('native-cli.test.cjs ok');

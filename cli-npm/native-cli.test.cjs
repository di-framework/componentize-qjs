'use strict';

const assert = require('node:assert/strict');
const {
  PACKAGE_NAME,
  WRAPPER_VERSION,
  platformPackageName,
  platformPackageVersion,
  platformPackageId,
  nativeCliPath,
} = require('./native-cli.cjs');

assert.equal(PACKAGE_NAME, '@di-framework/componentize-qjs');
assert.equal(WRAPPER_VERSION, '0.4.4-di.2');
assert.equal(platformPackageName('darwin'), '@di-framework/componentize-qjs-darwin');
assert.equal(platformPackageName('linux'), '@di-framework/componentize-qjs-linux');
assert.equal(platformPackageName('win32'), '@di-framework/componentize-qjs-win32');
assert.equal(platformPackageVersion('arm64'), '0.4.4-di.2-arm64');
assert.equal(platformPackageVersion('x64'), '0.4.4-di.2-x64');
assert.equal(
  platformPackageId('darwin', 'arm64'),
  '@di-framework/componentize-qjs-darwin@0.4.4-di.2-arm64',
);
assert.equal(
  platformPackageId('linux', 'x64'),
  '@di-framework/componentize-qjs-linux@0.4.4-di.2-x64',
);
assert.equal(nativeCliPath('plan9', 'x64'), undefined);
console.log('native-cli.test.cjs ok');

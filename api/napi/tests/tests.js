let slint = require("../index.js");
var assert = require('assert');

let componentCompiler = new slint.ComponentCompiler();

componentCompiler.setIncludePaths(["path/one/", "path/two/", "path/three/"]);
let includePaths = componentCompiler.includePaths();
assert.equal(includePaths.length, 3);
assert.equal(includePaths[0], "path/one/");
assert.equal(includePaths[1], "path/two/");
assert.equal(includePaths[2], "path/three/");

componentCompiler.setStyle("fluent");
assert.equal(componentCompiler.style(), "fluent");

let compiler_definition = componentCompiler.buildFromSource("export component Test {}", "");

assert.notEqual(compiler_definition, null);
assert(compiler_definition.name(), "Test");
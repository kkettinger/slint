let slint = require("../index.js");

let componentCompiler = new slint.ComponentCompiler();
componentCompiler.setStyle("fluent");
console.log(componentCompiler.style());
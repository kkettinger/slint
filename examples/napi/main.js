import { ComponentCompiler } from 'slint-ui';

let compiler = new ComponentCompiler();
let definition = compiler.buildFromPath("window.slint");
console.log(definition.name);
let instance = definition.create();
instance.run();
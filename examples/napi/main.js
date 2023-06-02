import { run } from 'slint-ui';

import * as fs from 'fs';
const fileName = "window.slint";
const fileData = fs.readFileSync(fileName, "utf8");

console.log(fileData);

run(fileData)
// @ts-check
const assert = require("assert");
const createFromBuffer = require("@dprint/formatter").createFromBuffer;
const getBuffer = require("./index").getBuffer;

const formatter = createFromBuffer(getBuffer());
const result = formatter.formatText("file.py", `print(   "Hi"   )`);

assert.strictEqual(result, `print("Hi")\n`);

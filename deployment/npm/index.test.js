// @ts-check
const assert = require("assert");
const createFromBuffer = require("@dprint/formatter").createFromBuffer;
const getBuffer = require("./index").getBuffer;

const formatter = createFromBuffer(getBuffer());
const result = formatter.formatText({
  filePath: "file.py",
  fileText: `print(   "Hi"   )`,
});

assert.strictEqual(result, `print("Hi")\n`);

const assertFormat = (config, fileText, expected) => {
  formatter.setConfig({}, config);
  assert.strictEqual(formatter.formatText({ filePath: "file.py", fileText }), expected);
};

const input = "import requests\nimport custom_pkg\nimport os\n";
assertFormat(
  { fixLintErrors: { unsortedImports: true } },
  input,
  "import os\n\nimport custom_pkg\nimport requests\n",
);

assertFormat(
  { fixLintErrors: { unsortedImports: true, knownFirstParty: ["custom_pkg"] } },
  input,
  "import os\n\nimport requests\n\nimport custom_pkg\n",
);

const versionInput = "import tomllib\nimport requests\nimport os\n";
assertFormat(
  { targetVersion: "py310", fixLintErrors: { unsortedImports: true } },
  versionInput,
  "import os\n\nimport requests\nimport tomllib\n",
);

assertFormat(
  { targetVersion: "py311", fixLintErrors: { unsortedImports: true } },
  versionInput,
  "import os\nimport tomllib\n\nimport requests\n",
);

formatter.setConfig({}, {
  fixLintErrors: { unsortedImports: true },
});
const omittedVersion = formatter.formatText({ filePath: "file.py", fileText: versionInput });

assertFormat(
  { targetVersion: "default", fixLintErrors: { unsortedImports: true } },
  versionInput,
  omittedVersion,
);

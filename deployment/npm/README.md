# @dprint/ruff

npm distribution of [dprint-plugin-ruff](https://github.com/dprint/dprint-plugin-ruff) which is an adapter plugin for [Ruff](https://github.com/astral-sh/ruff).

Use this with [@dprint/formatter](https://github.com/dprint/js-formatter) or just use @dprint/formatter and download the [dprint-plugin-ruff Wasm file](https://github.com/dprint/dprint-plugin-ruff/releases).

## Example

```ts
import { createFromBuffer } from "@dprint/formatter";
import { getBuffer } from "@dprint/ruff";

const formatter = createFromBuffer(getBuffer());

console.log(
  formatter.formatText("test.py", `print(  "Hi!")`),
);
```

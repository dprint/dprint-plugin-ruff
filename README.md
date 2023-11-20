# dprint-plugin-ruff

[![](https://img.shields.io/crates/v/dprint-plugin-ruff.svg)](https://crates.io/crates/dprint-plugin-ruff) [![CI](https://github.com/dprint/dprint-plugin-ruff/workflows/CI/badge.svg)](https://github.com/dprint/dprint-plugin-ruff/actions?query=workflow%3ACI)

Adapter for [Ruff](https://github.com/astral-sh/ruff) for use as a formatting plugin in [dprint](https://github.com/dprint/dprint).

Formats .py and .pyi files.

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

```shellsession
dprint config add ruff
```

## Configuration

To add configuration, specify a `"ruff"` key in your dprint.json:

```jsonc
{
  "ruff": {
    "indentStyle": "space",
    "lineLength": 100,
    "indentWidth": 2
  },
  "plugins": [
    // ...etc...
  ]
}
```

For an overview of the config, see https://dprint.dev/plugins/ruff/config/

## JS Formatting API

- [JS Formatter](https://github.com/dprint/js-formatter) - Browser/Deno and Node
- [npm package](https://www.npmjs.com/package/@dprint/ruff)

## Versioning

This repo automatically upgrades to the latest version of Ruff once a day. You can check which version of Ruff is being used by looking at the `tag` property in the `ruff_python_formatter` entry in the Cargo.toml file in this repo:

https://github.com/dprint/dprint-plugin-ruff/blob/main/Cargo.toml

At the moment, the version of this plugin does not reflect the version of Ruff. This is just in case there are any small bug fixes that need to be made as this plugin is quite new. After a while I'll try to match the versions.

extern crate dprint_development;
extern crate dprint_plugin_ruff;

use std::path::{Path, PathBuf};

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_ruff::configuration::resolve_config;
use dprint_plugin_ruff::configuration::{Configuration, FixLintErrors, TargetVersion};
use dprint_plugin_ruff::*;
use pretty_assertions::assert_eq;
use ruff_python_ast::PythonVersion;
use serde_json::json;

fn resolve_config_from_json(value: serde_json::Value) -> ResolveConfigurationResult<Configuration> {
  let config: ConfigKeyMap = serde_json::from_value(value).unwrap();
  resolve_config(config, &GlobalConfiguration::default())
}

fn format_resolved(file_path: &Path, source: &str, value: serde_json::Value) -> String {
  let result = resolve_config_from_json(value);
  ensure_no_diagnostics(&result.diagnostics);
  format_text(file_path, source, &result.config)
    .unwrap()
    .unwrap_or_else(|| source.to_string())
}

#[test]
fn test_specs() {
  let global_config = GlobalConfiguration::default();

  run_specs(
    &PathBuf::from("./tests/specs"),
    &ParseSpecOptions {
      default_file_name: "file.py",
    },
    &RunSpecsOptions {
      fix_failures: false,
      format_twice: true,
    },
    {
      let global_config = global_config.clone();
      move |file_path, file_text, spec_config| {
        let spec_config: ConfigKeyMap = serde_json::from_value(spec_config.clone().into()).unwrap();
        let config_result = resolve_config(spec_config, &global_config);
        ensure_no_diagnostics(&config_result.diagnostics);

        format_text(file_path, file_text, &config_result.config)
      }
    },
    move |_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing."),
  )
}

#[test]
fn fix_lint_error_diagnostics_are_nested() {
  let config: ConfigKeyMap = serde_json::from_value(json!({
    "fixLintErrors": {
      "unusedImport": {},
      "requiredImports": {},
    }
  }))
  .unwrap();

  let result = resolve_config(config, &GlobalConfiguration::default());
  let diagnostic_property_names: Vec<_> = result
    .diagnostics
    .iter()
    .map(|diagnostic| diagnostic.property_name.as_str())
    .collect();

  assert_eq!(
    diagnostic_property_names,
    vec!["fixLintErrors.unusedImport", "fixLintErrors.requiredImports"]
  );
}

#[test]
fn target_version_accepts_ruff_version_names() {
  for (name, version) in [
    ("py37", PythonVersion::PY37),
    ("py310", PythonVersion::PY310),
    ("py315", PythonVersion::PY315),
  ] {
    let result = resolve_config_from_json(json!({ "targetVersion": name }));
    assert!(result.diagnostics.is_empty(), "unexpected diagnostics for {name}");
    assert_eq!(result.config.target_version, Some(TargetVersion(version)));
  }
}

#[test]
fn target_version_rejects_names_ruff_does_not_accept() {
  // Ruff has no "auto" value, and the plugin does not infer a version from requires-python.
  // The bundled Ruff default is used by omitting the option.
  for value in ["auto", "default", "py3", "py3.10"] {
    let result = resolve_config_from_json(json!({ "targetVersion": value }));
    let properties: Vec<_> = result
      .diagnostics
      .iter()
      .map(|diagnostic| diagnostic.property_name.as_str())
      .collect();
    assert_eq!(properties, vec!["targetVersion"], "expected one diagnostic for {value}");
  }
}

#[test]
fn resolved_config_survives_serde_round_trip() {
  let result = resolve_config_from_json(json!({
    "targetVersion": "py310",
    "fixLintErrors": { "knownFirstParty": ["app.*"], "knownThirdParty": ["requests"] }
  }));
  let serialized = serde_json::to_value(&result.config).unwrap();
  assert_eq!(serialized["targetVersion"], json!("py310"));
  assert_eq!(serialized["fixLintErrors"]["knownFirstParty"], json!(["app.*"]));
  assert_eq!(serialized["fixLintErrors"]["knownThirdParty"], json!(["requests"]));

  let round_trip: Configuration = serde_json::from_value(serialized).unwrap();
  assert_eq!(round_trip.target_version, Some(TargetVersion(PythonVersion::PY310)));
  let lint = round_trip.fix_lint_errors.unwrap();
  assert_eq!(lint.known_first_party, Some(vec!["app.*".to_string()]));
  assert_eq!(lint.known_third_party, Some(vec!["requests".to_string()]));
}

#[test]
fn schema_target_versions_match_ruff() {
  // The schema is maintained by hand, so it is checked against the versions the bundled Ruff supports.
  let schema: serde_json::Value = serde_json::from_str(include_str!("../deployment/schema.json")).unwrap();
  let target_version = &schema["properties"]["targetVersion"];
  let versions = TargetVersion::all_names();

  assert_eq!(
    target_version["enum"],
    json!(versions),
    "deployment/schema.json targetVersion enum is out of sync with the bundled Ruff; set it to {versions:?}"
  );
}

#[test]
fn new_settings_report_invalid_values() {
  let config: ConfigKeyMap = serde_json::from_value(json!({
    "targetVersion": "py999",
    "fixLintErrors": {
      "knownFirstParty": ["[", false],
      "knownThirdParty": {}
    }
  }))
  .unwrap();
  let result = resolve_config(config, &GlobalConfiguration::default());
  let properties: Vec<_> = result
    .diagnostics
    .iter()
    .map(|diagnostic| diagnostic.property_name.as_str())
    .collect();
  assert_eq!(
    properties,
    vec![
      "fixLintErrors.knownFirstParty[0]",
      "fixLintErrors.knownFirstParty[1]",
      "fixLintErrors.knownThirdParty",
      "targetVersion"
    ]
  );

  let config: ConfigKeyMap = serde_json::from_value(json!({
    "targetVersion": ["py310"],
    "fixLintErrors": { "knownFirstParty": "app", "knownThirdParty": [null] }
  }))
  .unwrap();
  let result = resolve_config(config, &GlobalConfiguration::default());
  let properties: Vec<_> = result
    .diagnostics
    .iter()
    .map(|diagnostic| diagnostic.property_name.as_str())
    .collect();
  assert_eq!(
    properties,
    vec![
      "fixLintErrors.knownFirstParty",
      "fixLintErrors.knownThirdParty[0]",
      "targetVersion"
    ]
  );
}

#[test]
fn known_modules_section_precedence() {
  let source = "import os\nimport requests\n";
  let file_path = PathBuf::from("file.py");
  let format = |value| format_resolved(&file_path, source, value);

  let baseline = format(json!({ "fixLintErrors": { "unsortedImports": true } }));
  assert_eq!(baseline, "import os\n\nimport requests\n");
  assert_eq!(
    format(json!({
      "fixLintErrors": { "unsortedImports": true, "knownThirdParty": ["os"] }
    })),
    "import os\nimport requests\n"
  );
  assert_eq!(
    format(json!({
      "fixLintErrors": { "unsortedImports": true, "knownFirstParty": ["os"] }
    })),
    "import requests\n\nimport os\n"
  );
  // First-party classification takes precedence when the lists overlap, as in Ruff.
  assert_eq!(
    format(json!({
      "fixLintErrors": { "unsortedImports": true, "knownFirstParty": ["os"], "knownThirdParty": ["os"] }
    })),
    "import requests\n\nimport os\n"
  );
}

#[test]
fn duplicate_known_modules_are_accepted() {
  // Ruff only warns about repeated or overlapping entries. dprint treats
  // configuration diagnostics as fatal, so duplicates must not produce any.
  for value in [
    json!({ "fixLintErrors": { "knownFirstParty": ["os", "os"] } }),
    json!({ "fixLintErrors": { "knownThirdParty": ["os", "os"] } }),
    json!({ "fixLintErrors": { "knownFirstParty": ["os"], "knownThirdParty": ["os"] } }),
  ] {
    let result = resolve_config_from_json(value.clone());
    assert!(result.diagnostics.is_empty(), "unexpected diagnostics for {value}");
  }
}

#[test]
fn format_reports_invalid_known_module_pattern() {
  // A Configuration built without resolve_config can still carry an invalid glob.
  let config = Configuration {
    fix_lint_errors: Some(FixLintErrors {
      unsorted_imports: Some(true),
      known_third_party: Some(vec!["requests".to_string(), "[".to_string()]),
      ..Default::default()
    }),
    ..Default::default()
  };
  // the linter settings are resolved once per configuration, so every file must still get the error
  for _ in 0..2 {
    let error = format_text(&PathBuf::from("file.py"), "import os\n", &config).unwrap_err();
    assert!(
      error
        .to_string()
        .starts_with("Invalid fixLintErrors.knownThirdParty[1]: "),
      "unexpected error: {error}"
    );
  }
}

#[test]
fn global_line_width_is_clamped_and_respects_plugin_override() {
  let global_config = GlobalConfiguration {
    line_width: Some(80),
    ..Default::default()
  };

  assert_eq!(
    resolve_config(ConfigKeyMap::new(), &global_config).config.line_length,
    Some(80)
  );

  let oversized_global_config = GlobalConfiguration {
    line_width: Some(u32::MAX),
    ..Default::default()
  };
  assert_eq!(
    resolve_config(ConfigKeyMap::new(), &oversized_global_config)
      .config
      .line_length,
    Some(u16::MAX)
  );

  let plugin_config: ConfigKeyMap = serde_json::from_value(json!({ "lineLength": 100 })).unwrap();
  assert_eq!(
    resolve_config(plugin_config, &global_config).config.line_length,
    Some(100)
  );
}

#[test]
fn should_fail_on_parse_error_js() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.py"), "$*&(#*$&#", &config).unwrap_err();
  assert_eq!(err.to_string(), r#"Got unexpected token $ at byte range 0..1"#);
}

#[test]
fn docstring_code_line_width_validates_and_serializes() {
  for value in [json!(1), json!(30), json!(65535), json!("dynamic")] {
    let result = resolve_config_from_json(json!({ "docstringCodeLineWidth": value }));
    ensure_no_diagnostics(&result.diagnostics);
    let serialized = serde_json::to_value(&result.config).unwrap();
    assert_eq!(serialized["docstringCodeLineWidth"], value);
  }

  for value in [json!(0), json!(-1), json!(65536), json!("auto"), json!(true)] {
    let result = resolve_config_from_json(json!({ "docstringCodeLineWidth": value }));
    assert!(result.config.docstring_code_line_width.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].property_name, "docstringCodeLineWidth");
  }
}

#[test]
fn range_inside_statement_formats_only_that_statement() {
  let source = "x  =  1\ny  =  [1,2]\nz  =  3\n";
  let range = range_of(source, "[1,2]");
  assert_eq!(
    format_range(source, range).as_deref(),
    Some("x  =  1\ny = [1, 2]\nz  =  3\n")
  );
}

#[test]
fn range_inside_nested_statement_keeps_indentation() {
  let source = "def f():\n    x  =  1\n    y  =  2\n";
  let range = range_of(source, "y  =");
  assert_eq!(
    format_range(source, range).as_deref(),
    Some("def f():\n    x  =  1\n    y = 2\n")
  );
}

#[test]
fn range_spanning_statements_formats_each_statement() {
  let source = "a  =  1\nb  =  2\nc  =  3\nd  =  4\n";
  let range = source.find("=  2").unwrap()..source.find("=  3").unwrap();
  assert_eq!(
    format_range(source, range).as_deref(),
    Some("a  =  1\nb = 2\nc = 3\nd  =  4\n")
  );
}

#[test]
fn already_formatted_range_is_unchanged() {
  let source = "x  =  1\ny = [1, 2]\nz  =  3\n";
  let range = range_of(source, "y = [1, 2]");
  assert_eq!(format_range(source, range), None);
}

#[test]
fn empty_range_is_unchanged() {
  let source = "x  =  1\n";
  assert_eq!(format_range(source, 3..3), None);
}

#[test]
fn range_after_multi_byte_chars() {
  let source = "s  =  'héllo 😀'\nx  =  [1,2]\n";
  let range = range_of(source, "[1,2]");
  assert_eq!(
    format_range(source, range).as_deref(),
    Some("s  =  'héllo 😀'\nx = [1, 2]\n")
  );
}

#[test]
fn range_keeps_bom() {
  let source = "\u{FEFF}x  =  1\ny  =  2\n";
  let range = range_of(source, "y  =  2");
  assert_eq!(format_range(source, range).as_deref(), Some("\u{FEFF}x  =  1\ny = 2\n"));
  let range = 0..source.find('x').unwrap() + 1;
  assert_eq!(format_range(source, range).as_deref(), Some("\u{FEFF}x = 1\ny  =  2\n"));
}

#[test]
fn range_off_char_boundaries_is_widened() {
  let source = "s  =  '😀'\nx  =  1\n";
  let emoji_start = source.find('😀').unwrap();
  // starts and ends inside the 4 byte emoji
  let range = emoji_start + 1..emoji_start + 2;
  assert_eq!(format_range(source, range).as_deref(), Some("s = \"😀\"\nx  =  1\n"));
}

#[test]
fn range_past_end_of_text_is_clamped() {
  let source = "x  =  1\ny  =  2\n";
  let range = source.find('y').unwrap()..source.len() + 10;
  assert_eq!(format_range(source, range).as_deref(), Some("x  =  1\ny = 2\n"));
}

#[test]
fn range_does_not_apply_lint_fixes() {
  let source = "import os\nx  =  1\n";
  let result = resolve_config_from_json(json!({ "fixLintErrors": { "unusedImport": true } }));
  ensure_no_diagnostics(&result.diagnostics);
  let range = range_of(source, "x  =  1");
  let formatted = format_text_range(&PathBuf::from("file.py"), source, range, &result.config).unwrap();
  assert_eq!(formatted.as_deref(), Some("import os\nx = 1\n"));
}

#[test]
fn range_in_stub_file() {
  let source = "def f(a:int)->int: ...\ndef g(b:str)->str: ...\n";
  let range = range_of(source, "b:str");
  let formatted = format_text_range(&PathBuf::from("file.pyi"), source, range, &Configuration::default()).unwrap();
  assert_eq!(
    formatted.as_deref(),
    Some("def f(a:int)->int: ...\ndef g(b: str) -> str: ...\n")
  );
}

#[test]
fn range_in_notebook_is_unchanged() {
  let source = r#"{"cells": [], "metadata": {}, "nbformat": 4, "nbformat_minor": 2}"#;
  let formatted = format_text_range(
    &PathBuf::from("file.ipynb"),
    source,
    0..source.len(),
    &Configuration::default(),
  )
  .unwrap();
  assert_eq!(formatted, None);
}

fn format_range(source: &str, range: std::ops::Range<usize>) -> Option<String> {
  format_text_range(&PathBuf::from("file.py"), source, range, &Configuration::default()).unwrap()
}

fn range_of(source: &str, needle: &str) -> std::ops::Range<usize> {
  let start = source.find(needle).unwrap();
  start..start + needle.len()
}

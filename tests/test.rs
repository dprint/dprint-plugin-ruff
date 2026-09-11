extern crate dprint_development;
extern crate dprint_plugin_ruff;

use std::path::PathBuf;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_ruff::configuration::Configuration;
use dprint_plugin_ruff::configuration::resolve_config;
use dprint_plugin_ruff::*;
use pretty_assertions::assert_eq;
use serde_json::json;

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

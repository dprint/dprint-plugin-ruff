extern crate dprint_development;
extern crate dprint_plugin_ruff;

use std::path::PathBuf;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_ruff::configuration::resolve_config;
use dprint_plugin_ruff::configuration::Configuration;
use dprint_plugin_ruff::*;
use pretty_assertions::assert_eq;

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

        format_text(file_path, &file_text, &config_result.config)
      }
    },
    move |_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing."),
  )
}

#[test]
fn should_fail_on_parse_error_js() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.py"), "$*&(#*$&#", &config).unwrap_err();
  assert_eq!(err.to_string(), r#"Got unexpected token $ at byte range 0..1"#);
}

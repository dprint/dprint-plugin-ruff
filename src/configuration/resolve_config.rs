use super::LineEnding;
use super::{Configuration, FixLintErrors};
use dprint_core::configuration::*;
use ruff_python_semantic::NameImports;
use serde::de::{Deserialize, value::StrDeserializer};

/// Resolves configuration from a collection of key value strings.
///
/// # Example
///
/// ```
/// use dprint_core::configuration::ConfigKeyMap;
/// use dprint_core::configuration::resolve_global_config;
/// use dprint_plugin_ruff::configuration::resolve_config;
///
/// let mut config_map = ConfigKeyMap::new(); // get a collection of key value pairs from somewhere
/// let global_config_result = resolve_global_config(&mut config_map);
///
/// // check global_config_result.diagnostics here...
///
/// let config_result = resolve_config(
///     config_map,
///     &global_config_result.config
/// );
///
/// // check config_result.diagnostics here and use config_result.config
/// ```
pub fn resolve_config(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
  let mut diagnostics = Vec::new();
  let mut config = config;
  let fix_lint_errors = get_nullable_object(&mut config, "fixLintErrors", &mut diagnostics).map(|mut lint_config| {
    let mut fix_lint_errors_diagnostics = Vec::new();
    let resolved_config = FixLintErrors {
      unused_import: get_nullable_value(&mut lint_config, "unusedImport", &mut fix_lint_errors_diagnostics),
      unsorted_imports: get_nullable_value(&mut lint_config, "unsortedImports", &mut fix_lint_errors_diagnostics),
      missing_required_import: get_nullable_value(
        &mut lint_config,
        "missingRequiredImport",
        &mut fix_lint_errors_diagnostics,
      ),
      required_imports: get_nullable_required_imports(&mut lint_config, &mut fix_lint_errors_diagnostics),
    };

    fix_lint_errors_diagnostics.extend(get_unknown_property_diagnostics(lint_config));
    diagnostics.extend(fix_lint_errors_diagnostics.into_iter().map(|diagnostic| {
      dprint_core::configuration::ConfigurationDiagnostic {
        property_name: format!("fixLintErrors.{}", diagnostic.property_name),
        message: diagnostic.message,
      }
    }));

    resolved_config
  });

  let resolved_config = Configuration {
    line_length: get_nullable_value(&mut config, "lineLength", &mut diagnostics).or(
      global_config
        .line_width
        .map(|l| std::cmp::min(u16::MAX as u32, l) as u16),
    ),
    indent_width: get_nullable_value(&mut config, "indentWidth", &mut diagnostics).or(global_config.indent_width),
    indent_style: get_nullable_value(&mut config, "indentStyle", &mut diagnostics),
    line_ending: get_nullable_value(&mut config, "lineEnding", &mut diagnostics).or(global_config.new_line_kind.map(
      |l| match l {
        // not ideal
        NewLineKind::Auto | NewLineKind::LineFeed => LineEnding::LineFeed,
        NewLineKind::CarriageReturnLineFeed => LineEnding::CarriageReturnLineFeed,
      },
    )),
    quote_style: get_nullable_value(&mut config, "quoteStyle", &mut diagnostics),
    skip_magic_trailing_comma: get_nullable_value(&mut config, "skipMagicTrailingComma", &mut diagnostics),
    preview: get_nullable_value(&mut config, "preview", &mut diagnostics),
    fix_lint_errors,
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

fn get_nullable_object(
  config: &mut ConfigKeyMap,
  key: &str,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<ConfigKeyMap> {
  match config.shift_remove(key) {
    Some(ConfigKeyValue::Object(value)) => Some(value),
    Some(ConfigKeyValue::Null) | None => None,
    Some(_) => {
      diagnostics.push(ConfigurationDiagnostic {
        property_name: key.to_string(),
        message: "Expected an object.".to_string(),
      });
      None
    }
  }
}

fn get_nullable_required_imports(
  config: &mut ConfigKeyMap,
  diagnostics: &mut Vec<ConfigurationDiagnostic>,
) -> Option<Vec<String>> {
  let required_imports = get_nullable_vec(
    config,
    "requiredImports",
    |value, index, diagnostics| match value.into_string() {
      Some(value) => Some(value),
      None => {
        diagnostics.push(ConfigurationDiagnostic {
          property_name: format!("requiredImports[{index}]"),
          message: "Expected a string.".to_string(),
        });
        None
      }
    },
    diagnostics,
  );

  if let Some(required_imports) = required_imports.as_ref() {
    for (index, required_import) in required_imports.iter().enumerate() {
      if let Err(error) = NameImports::deserialize(StrDeserializer::<serde::de::value::Error>::new(required_import)) {
        diagnostics.push(ConfigurationDiagnostic {
          property_name: format!("requiredImports[{index}]"),
          message: format!("Invalid import statement: {error}"),
        });
      }
    }
  }

  required_imports
}

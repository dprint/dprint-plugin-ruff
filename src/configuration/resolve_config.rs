use super::Configuration;
use super::LineEnding;
use dprint_core::configuration::*;

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

  let resolved_config = Configuration {
    line_length: get_nullable_value(&mut config, "lineLength", &mut diagnostics).or(
      global_config
        .line_width
        .map(|l| std::cmp::max(u16::MAX as u32, l) as u16),
    ),
    indent_width: get_nullable_value(&mut config, "indentWidth", &mut diagnostics).or(global_config.indent_width),
    indent_style: get_nullable_value(&mut config, "indentStyle", &mut diagnostics),
    line_ending: get_nullable_value(&mut config, "lineEnding", &mut diagnostics).or(global_config.new_line_kind.map(
      |l| match l {
        // not ideal
        NewLineKind::Auto | NewLineKind::LineFeed | NewLineKind::System => LineEnding::LineFeed,
        NewLineKind::CarriageReturnLineFeed => LineEnding::CarriageReturnLineFeed,
      },
    )),
    quote_style: get_nullable_value(&mut config, "quoteStyle", &mut diagnostics),
    skip_magic_trailing_comma: get_nullable_value(&mut config, "skipMagicTrailingComma", &mut diagnostics),
    preview: get_nullable_value(&mut config, "preview", &mut diagnostics),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

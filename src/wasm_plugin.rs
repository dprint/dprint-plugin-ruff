use super::configuration::{resolve_config, Configuration};

use dprint_core::configuration::{ConfigKeyMap, GlobalConfiguration};
use dprint_core::generate_plugin_code;
use dprint_core::plugins::CheckConfigUpdatesMessage;
use dprint_core::plugins::ConfigChange;
use dprint_core::plugins::FileMatchingInfo;
use dprint_core::plugins::FormatError;
use dprint_core::plugins::FormatResult;
use dprint_core::plugins::PluginInfo;
use dprint_core::plugins::PluginResolveConfigurationResult;
use dprint_core::plugins::SyncFormatRequest;
use dprint_core::plugins::SyncHostFormatRequest;
use dprint_core::plugins::SyncPluginHandler;

struct RuffPluginHandler;

impl SyncPluginHandler<Configuration> for RuffPluginHandler {
  fn resolve_config(
    &mut self,
    config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
  ) -> PluginResolveConfigurationResult<Configuration> {
    let result = resolve_config(config, global_config);
    PluginResolveConfigurationResult {
      config: result.config,
      diagnostics: result.diagnostics,
      file_matching: FileMatchingInfo {
        file_extensions: vec![
          "py".to_string(),
          "pyi".to_string(),
          // disabled for a bit because I found it breaks notebooks in vscode
          // "ipynb".to_string()
        ],
        file_names: vec![],
      },
    }
  }

  fn plugin_info(&mut self) -> PluginInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    PluginInfo {
      name: env!("CARGO_PKG_NAME").to_string(),
      version: version.clone(),
      config_key: "ruff".to_string(),
      help_url: "https://dprint.dev/plugins/ruff".to_string(),
      config_schema_url: format!(
        "https://plugins.dprint.dev/dprint/dprint-plugin-ruff/{}/schema.json",
        version
      ),
      update_url: Some("https://plugins.dprint.dev/dprint/dprint-plugin-ruff/latest.json".to_string()),
    }
  }

  fn license_text(&mut self) -> String {
    std::str::from_utf8(include_bytes!("../LICENSE")).unwrap().into()
  }

  fn check_config_updates(&self, _message: CheckConfigUpdatesMessage) -> Result<Vec<ConfigChange>, FormatError> {
    Ok(Vec::new())
  }

  fn format(
    &mut self,
    request: SyncFormatRequest<Configuration>,
    _format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
  ) -> FormatResult {
    if request.range.is_some() {
      return Ok(None); // not supported
    }

    let file_text = String::from_utf8(request.file_bytes).map_err(FormatError::new)?;
    super::format_text(request.file_path, &file_text, request.config)
      .map(|maybe_text| maybe_text.map(|t| t.into_bytes()))
      .map_err(FormatError::new)
  }
}

generate_plugin_code!(RuffPluginHandler, RuffPluginHandler);

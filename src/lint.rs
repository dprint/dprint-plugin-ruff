use std::collections::BTreeSet;
use std::path::Path;

use crate::configuration::Configuration;
use anyhow::{Result, anyhow};
use ruff_linter::line_width::{IndentWidth, LineLength};
use ruff_linter::linter::lint_fix;
use ruff_linter::registry::Rule;
use ruff_linter::settings::LinterSettings;
use ruff_linter::settings::flags::Noqa;
use ruff_linter::settings::rule_table::RuleTable;
use ruff_linter::settings::types::UnsafeFixes;
use ruff_linter::source_kind::SourceKind;
use ruff_python_ast::PySourceType;
use ruff_python_semantic::{NameImport, NameImports};
use serde::de::{Deserialize, value::StrDeserializer};

pub(crate) fn apply_lint_fixes(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>> {
  let enabled_rules = resolve_enabled_rules(config);
  if enabled_rules.is_empty() {
    return Ok(None);
  }

  initialize_ruff_linter();

  let source_type = PySourceType::from(file_path);
  let source_kind = SourceKind::Python {
    code: input_text.to_owned(),
    is_stub: source_type.is_stub(),
  };
  let settings = resolve_linter_settings(config, &enabled_rules)?;
  let result = lint_fix(
    file_path,
    None,
    Noqa::Enabled,
    UnsafeFixes::default(),
    &settings,
    &source_kind,
    source_type,
  )?;
  let transformed = result.transformed.source_code();

  if transformed == input_text {
    Ok(None)
  } else {
    Ok(Some(transformed.to_owned()))
  }
}

fn resolve_enabled_rules(config: &Configuration) -> Vec<Rule> {
  let mut rules = Vec::new();
  let Some(fix_lint_errors) = config.fix_lint_errors.as_ref() else {
    return rules;
  };

  if fix_lint_errors.unused_import == Some(true) {
    rules.push(Rule::UnusedImport);
  }
  if fix_lint_errors.unsorted_imports == Some(true) {
    rules.push(Rule::UnsortedImports);
  }
  if fix_lint_errors.missing_required_import == Some(true) {
    rules.push(Rule::MissingRequiredImport);
  }
  rules
}

fn resolve_linter_settings(config: &Configuration, enabled_rules: &[Rule]) -> Result<LinterSettings> {
  let mut rules = RuleTable::empty();
  for &rule in enabled_rules {
    rules.enable(rule, true);
  }

  let mut settings = LinterSettings {
    rules,
    preview: config.preview.unwrap_or(false).into(),
    line_length: config
      .line_length
      .and_then(|value| LineLength::try_from(value).ok())
      .unwrap_or_default(),
    tab_size: config
      .indent_width
      .and_then(std::num::NonZeroU8::new)
      .map(IndentWidth::from)
      .unwrap_or_default(),
    ..LinterSettings::default()
  };

  settings.isort.required_imports = resolve_required_imports(config)?;

  Ok(settings)
}

fn resolve_required_imports(config: &Configuration) -> Result<BTreeSet<NameImport>> {
  let Some(required_imports) = config
    .fix_lint_errors
    .as_ref()
    .and_then(|config| config.required_imports.as_ref())
  else {
    return Ok(BTreeSet::new());
  };

  required_imports
    .iter()
    .try_fold(BTreeSet::new(), |mut resolved_imports, required_import| {
      let imports = NameImports::deserialize(StrDeserializer::<serde::de::value::Error>::new(required_import))
        .map_err(|error| anyhow!("Invalid required import {required_import:?}: {error}"))?;
      resolved_imports.extend(imports.into_imports());
      Ok(resolved_imports)
    })
}

#[cfg(target_family = "wasm")]
fn initialize_ruff_linter() {
  static INIT: std::sync::Once = std::sync::Once::new();

  INIT.call_once(|| {
    #[expect(unsafe_code)]
    unsafe extern "C" {
      fn __wasm_call_ctors();
    }

    #[expect(unsafe_code)]
    unsafe {
      __wasm_call_ctors();
    }
  });
}

#[cfg(not(target_family = "wasm"))]
const fn initialize_ruff_linter() {}

use std::num::NonZeroU16;
use std::num::NonZeroU8;
use std::path::Path;

use crate::configuration::Configuration;
use anyhow::Result;
use ruff_formatter::printer::LineEnding;
use ruff_python_formatter::format_module_source;
use ruff_python_formatter::PyFormatOptions;
use ruff_python_formatter::QuoteStyle;

pub fn format_text(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>> {
  let options = resolve_options(file_path, config);
  let printed = format_module_source(input_text, options)?;
  let code = printed.into_code();
  if code == input_text {
    Ok(None)
  } else {
    Ok(Some(code))
  }
}

fn resolve_options(file_path: &Path, config: &Configuration) -> PyFormatOptions {
  let mut options = PyFormatOptions::from_extension(file_path);
  if let Some(indent_style) = &config.indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Space => ruff_formatter::IndentStyle::Space,
      crate::configuration::IndentStyle::Tab => ruff_formatter::IndentStyle::Tab,
    });
  }
  if let Some(line_width) = config.line_length.and_then(NonZeroU16::new) {
    options = options.with_line_width(line_width.into());
  }
  if let Some(indent_width) = config.indent_width.and_then(NonZeroU8::new) {
    options = options.with_indent_width(indent_width.into());
  }
  if let Some(line_ending) = config.line_ending {
    options = options.with_line_ending(match line_ending {
      crate::configuration::LineEnding::LineFeed => LineEnding::LineFeed,
      crate::configuration::LineEnding::CarriageReturnLineFeed => LineEnding::CarriageReturnLineFeed,
    });
  }
  if let Some(quote_style) = config.quote_style {
    options = options.with_quote_style(match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    })
  }
  if let Some(value) = config.skip_magic_trailing_comma {
    options = options.with_magic_trailing_comma(match value {
      true => ruff_python_formatter::MagicTrailingComma::Respect,
      false => ruff_python_formatter::MagicTrailingComma::Ignore,
    });
  }
  if let Some(value) = config.preview {
    options = options.with_preview(match value {
      true => ruff_python_formatter::PreviewMode::Enabled,
      false => ruff_python_formatter::PreviewMode::Disabled,
    });
  }
  options
}

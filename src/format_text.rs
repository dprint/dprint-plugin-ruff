use std::num::NonZeroU8;
use std::num::NonZeroU16;
use std::ops::Range;
use std::path::Path;

use crate::configuration::Configuration;
use anyhow::Result;
use ruff_formatter::printer::LineEnding;
use ruff_python_formatter::PyFormatOptions;
use ruff_python_formatter::QuoteStyle;
use ruff_python_formatter::format_module_source;
use ruff_python_formatter::format_range;
use ruff_text_size::TextRange;
use ruff_text_size::TextSize;

pub fn format_text(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>> {
  let linted_text = crate::lint::apply_lint_fixes(file_path, input_text, config)?;
  let source_text = linted_text.as_deref().unwrap_or(input_text);
  let options = resolve_options(file_path, config);
  let printed = format_module_source(source_text, options)?;
  let code = printed.into_code();
  if code == input_text { Ok(None) } else { Ok(Some(code)) }
}

/// Formats only the part of the text within the provided byte range.
///
/// Ruff widens the range to the logical lines it touches and the text outside of those is left as
/// it was. Lint fixes aren't applied because they can change code outside of the range, and
/// notebooks are left unformatted because Ruff can't range format them.
pub fn format_text_range(
  file_path: &Path,
  input_text: &str,
  range: Range<usize>,
  config: &Configuration,
) -> Result<Option<String>> {
  let options = resolve_options(file_path, config);
  if options.source_type().is_ipynb() {
    return Ok(None);
  }

  // Ruff's range formatting panics when the text starts with a BOM, so format the text after it
  // and keep the BOM as it was
  let text = input_text.strip_prefix('\u{FEFF}').unwrap_or(input_text);
  let bom = &input_text[..input_text.len() - text.len()];
  let range = range.start.saturating_sub(bom.len())..range.end.saturating_sub(bom.len());

  let range = clamp_to_char_boundaries(text, range);
  let range = TextRange::new(TextSize::try_from(range.start)?, TextSize::try_from(range.end)?);
  let printed = format_range(text, range, options)?;
  let source_range = Range::<usize>::from(printed.source_range());
  let code = printed.as_code();

  let mut formatted = String::with_capacity(input_text.len() - source_range.len() + code.len());
  formatted.push_str(bom);
  formatted.push_str(&text[..source_range.start]);
  formatted.push_str(code);
  formatted.push_str(&text[source_range.end..]);
  if formatted == input_text {
    Ok(None)
  } else {
    Ok(Some(formatted))
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
      crate::configuration::LineEnding::CarriageReturn => LineEnding::CarriageReturn,
    });
  }
  if let Some(quote_style) = config.quote_style {
    options = options.with_quote_style(match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
      crate::configuration::QuoteStyle::Preserve => QuoteStyle::Preserve,
    })
  }
  if let Some(value) = config.skip_magic_trailing_comma {
    options = options.with_magic_trailing_comma(match value {
      true => ruff_python_formatter::MagicTrailingComma::Ignore,
      false => ruff_python_formatter::MagicTrailingComma::Respect,
    });
  }
  if let Some(value) = config.docstring_code_format {
    options = options.with_docstring_code(match value {
      true => ruff_python_formatter::DocstringCode::Enabled,
      false => ruff_python_formatter::DocstringCode::Disabled,
    });
  }
  if let Some(value) = config.docstring_code_line_width {
    options = options.with_docstring_code_line_width(match value {
      crate::configuration::DocstringCodeLineWidth::Fixed(width) => {
        ruff_python_formatter::DocstringCodeLineWidth::Fixed(width.into())
      }
      crate::configuration::DocstringCodeLineWidth::Dynamic(_) => {
        ruff_python_formatter::DocstringCodeLineWidth::Dynamic
      }
    });
  }
  if let Some(value) = config.nested_string_quote_style {
    options = options.with_nested_string_quote_style(match value {
      crate::configuration::NestedStringQuoteStyle::Alternating => {
        ruff_python_formatter::NestedStringQuoteStyle::Alternating
      }
      crate::configuration::NestedStringQuoteStyle::Preferred => {
        ruff_python_formatter::NestedStringQuoteStyle::Preferred
      }
    });
  }
  if let Some(value) = config.preview {
    options = options.with_preview(match value {
      true => ruff_python_formatter::PreviewMode::Enabled,
      false => ruff_python_formatter::PreviewMode::Disabled,
    });
  }
  if let Some(target_version) = config.target_version {
    options = options.with_target_version(target_version.0);
  }
  options
}

/// Clamps the range to the text and widens it to the nearest char boundaries.
fn clamp_to_char_boundaries(text: &str, range: Range<usize>) -> Range<usize> {
  let mut end = range.end.min(text.len());
  let mut start = range.start.min(end);
  while !text.is_char_boundary(start) {
    start -= 1;
  }
  while !text.is_char_boundary(end) {
    end += 1;
  }
  start..end
}

#[cfg(test)]
mod test {
  #[test]
  fn strips_bom() {
    let input = "\u{FEFF}print('hello')";
    let config = crate::configuration::Configuration::default();
    let file_path = std::path::Path::new("test.py");
    let result = super::format_text(file_path, input, &config).unwrap().unwrap();
    assert_eq!(result, "print(\"hello\")\n");
  }
}

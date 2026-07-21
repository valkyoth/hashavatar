use crate::CatError;

const MAX_ID_PREFIX_BYTES: usize = 64;
const MAX_TITLE_BYTES: usize = 256;
const MAX_DESCRIPTION_BYTES: usize = 512;

/// Selects complete SVG document or embeddable fragment output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgMode {
    /// Emit a complete `<svg>` document with accessibility metadata.
    Document,
    /// Emit a `<g>` fragment with deterministic prefixed identifiers.
    Fragment,
}

/// Validated options for deterministic SVG serialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgOptions<'a> {
    pub(super) mode: SvgMode,
    pub(super) id_prefix: &'a str,
    pub(super) title: Option<&'a str>,
    pub(super) description: Option<&'a str>,
}

impl<'a> SvgOptions<'a> {
    /// Creates complete-document options with escaped accessibility text.
    pub fn document(
        id_prefix: &'a str,
        title: &'a str,
        description: &'a str,
    ) -> Result<Self, CatError> {
        let options = Self {
            mode: SvgMode::Document,
            id_prefix,
            title: Some(title),
            description: Some(description),
        };
        options.validate()?;
        Ok(options)
    }

    /// Creates an embeddable fragment. Accessibility belongs to its host.
    pub fn fragment(id_prefix: &'a str) -> Result<Self, CatError> {
        let options = Self {
            mode: SvgMode::Fragment,
            id_prefix,
            title: None,
            description: None,
        };
        options.validate()?;
        Ok(options)
    }

    /// Returns the selected output mode.
    pub const fn mode(self) -> SvgMode {
        self.mode
    }

    /// Returns the validated identifier prefix.
    pub const fn id_prefix(self) -> &'a str {
        self.id_prefix
    }

    pub(super) fn validate(self) -> Result<(), CatError> {
        let mut chars = self.id_prefix.chars();
        let valid_first = chars
            .next()
            .is_some_and(|value| value.is_ascii_alphabetic());
        if !valid_first
            || self.id_prefix.len() > MAX_ID_PREFIX_BYTES
            || !chars.all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
        {
            return Err(CatError::InvalidSvgOptions);
        }
        match self.mode {
            SvgMode::Document => {
                let title = self.title.ok_or(CatError::InvalidSvgOptions)?;
                let description = self.description.ok_or(CatError::InvalidSvgOptions)?;
                if title.len() > MAX_TITLE_BYTES
                    || description.len() > MAX_DESCRIPTION_BYTES
                    || !title.chars().all(valid_xml_character)
                    || !description.chars().all(valid_xml_character)
                {
                    return Err(CatError::InvalidSvgOptions);
                }
            }
            SvgMode::Fragment if self.title.is_some() || self.description.is_some() => {
                return Err(CatError::InvalidSvgOptions);
            }
            SvgMode::Fragment => {}
        }
        Ok(())
    }
}

fn valid_xml_character(value: char) -> bool {
    matches!(value, '\u{9}' | '\u{a}' | '\u{d}')
        || ('\u{20}'..='\u{d7ff}').contains(&value)
        || ('\u{e000}'..='\u{fffd}').contains(&value)
        || ('\u{10000}'..='\u{10ffff}').contains(&value)
}

impl Default for SvgOptions<'static> {
    fn default() -> Self {
        Self {
            mode: SvgMode::Document,
            id_prefix: "hashavatar",
            title: Some("Hashavatar Cat"),
            description: Some("Deterministic procedural Cat avatar"),
        }
    }
}

//! Classification result types shared by the line classifier and its
//! consumers: [`Marker`], [`ValueKind`] and [`LineKind`].

/// Marker shape on a `key:` line, matching `ktav`'s `Separator` enum.
///
/// Spec 0.5.0: `:i` and `:f` typed markers are removed. Only `Plain` (`:`)
/// and `Raw` (`::`) remain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Plain `:`.
    Plain,
    /// `::` — raw / literal-string body.
    Raw,
}

impl Marker {
    /// Byte length of the marker text on the line.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> usize {
        match self {
            Marker::Plain => 1,
            Marker::Raw => 2,
        }
    }
}

/// What kind of value follows a marker (or stands alone as an array item).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// `null`.
    Null,
    /// `true` / `false`.
    Bool,
    /// Looks numeric (the surface form, no type marker).
    Number,
    /// Anything else — also the body of `::` raw markers.
    String,
    /// Compound opener / inline-empty: `{`, `[`, `(`, `((`, `{}`, `[]`, `()`.
    CompoundOpen,
    /// Lone closer line: `}`, `]`, `)`.
    CompoundClose,
}

/// One classified line.
#[derive(Debug, Clone)]
pub enum LineKind<'a> {
    /// Blank or whitespace-only.
    Blank,
    /// `# ...` line.
    Comment {
        /// Column of the `#`.
        start: u32,
        /// Trimmed-trailing length.
        length: u32,
    },
    /// Lone `}` / `]` / `)` closer.
    CloseBrace {
        /// Column of the closer.
        start: u32,
    },
    /// `:: value` literal-string array item.
    RawArrayItem {
        /// Column of the `::`.
        marker_start: u32,
        /// Value span (column + length); zero-length if no value.
        value_start: u32,
        value_length: u32,
    },
    /// `key{:|::|:i|:f} value` — the workhorse line.
    Pair {
        /// Column of the first byte of `key`.
        key_start: u32,
        /// Length in bytes of `key` (the dotted path is one slice — we
        /// expose dot-segment splitting via [`crate::tokens::split_dotted`] when needed).
        key_length: u32,
        /// Column of the marker's first byte.
        marker_start: u32,
        marker: Marker,
        /// Value span. `value_length == 0` ⇒ no value on this line
        /// (compound opener will be on the next line, etc.).
        value_start: u32,
        value_length: u32,
        /// Pre-computed kind. For `Raw` markers this is always
        /// [`ValueKind::String`]; for typed markers always
        /// [`ValueKind::Number`]; for plain markers it is the result of
        /// [`crate::tokens::classify_value`] applied to the value slice.
        value_kind: ValueKind,
        /// Borrowed slice of the value text (already trimmed).
        value_text: &'a str,
    },
    /// Bare item line inside an array (no `:` on the line).
    ArrayItem {
        start: u32,
        length: u32,
        kind: ValueKind,
    },
}

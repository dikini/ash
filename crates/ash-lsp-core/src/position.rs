//! Shared position and token-extraction helpers for LSP features.
//!
//! All line/column values use the LSP convention: **0-indexed**.

/// Build a table of byte offsets for the start of each line.
#[must_use]
pub fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (idx, ch) in source.char_indices() {
        if ch == '\n' {
            starts.push(idx + 1);
        }
    }
    starts
}

/// Convert a 0-indexed (line, column) pair to a byte offset.
#[must_use]
pub fn offset_from_line_col(source: &str, line: u32, col: u32) -> Option<usize> {
    let starts = line_starts(source);
    let line = usize::try_from(line).ok()?;
    let col = usize::try_from(col).ok()?;
    let start = *starts.get(line)?;
    let end = if line + 1 < starts.len() {
        starts[line + 1].saturating_sub(1)
    } else {
        source.len()
    };
    Some((start + col).min(end))
}

/// Convert a byte offset to a 0-indexed (line, column) pair.
#[must_use]
pub fn line_col_from_offset(source: &str, offset: usize) -> Option<(u32, u32)> {
    if offset > source.len() {
        return None;
    }
    let starts = line_starts(source);
    let line = match starts.binary_search(&offset) {
        Ok(idx) => idx,
        Err(idx) => idx - 1,
    };
    let col = offset - starts[line];
    Some((u32::try_from(line).ok()?, u32::try_from(col).ok()?))
}

#[must_use]
pub const fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Extract the identifier token at `offset` (byte offset) in `source`.
///
/// Returns `None` when the cursor is not inside an identifier.
#[must_use]
pub fn token_at_offset(source: &str, offset: usize) -> Option<&str> {
    if offset > source.len() {
        return None;
    }

    let mut selected = None;
    for (idx, ch) in source.char_indices() {
        let end = idx + ch.len_utf8();
        if offset >= idx && offset < end {
            selected = Some((idx, ch));
            break;
        }
    }

    let (idx, ch) = selected?;
    if !is_ident_char(ch) {
        return None;
    }

    let mut start = idx;
    for (prev_idx, prev_ch) in source[..idx].char_indices().rev() {
        if is_ident_char(prev_ch) {
            start = prev_idx;
        } else {
            break;
        }
    }

    let mut end = idx + ch.len_utf8();
    for (_next_rel, next_ch) in source[end..].char_indices() {
        if is_ident_char(next_ch) {
            end += next_ch.len_utf8();
        } else {
            break;
        }
    }

    source.get(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_starts() {
        assert_eq!(line_starts("hello"), vec![0]);
        assert_eq!(line_starts("hello\nworld"), vec![0, 6]);
        assert_eq!(line_starts("a\nb\nc"), vec![0, 2, 4]);
    }

    #[test]
    fn test_offset_from_line_col() {
        let src = "hello\nworld\n";
        assert_eq!(offset_from_line_col(src, 0, 0), Some(0));
        assert_eq!(offset_from_line_col(src, 0, 3), Some(3));
        assert_eq!(offset_from_line_col(src, 1, 0), Some(6));
        assert_eq!(offset_from_line_col(src, 1, 2), Some(8));
    }

    #[test]
    fn test_line_col_from_offset() {
        let src = "hello\nworld\n";
        assert_eq!(line_col_from_offset(src, 0), Some((0, 0)));
        assert_eq!(line_col_from_offset(src, 3), Some((0, 3)));
        assert_eq!(line_col_from_offset(src, 6), Some((1, 0)));
        assert_eq!(line_col_from_offset(src, 8), Some((1, 2)));
    }

    #[test]
    fn test_token_at_offset() {
        let src = "fn helper(x: Int) -> String { x }";
        assert_eq!(token_at_offset(src, 0), Some("fn"));
        assert_eq!(token_at_offset(src, 3), Some("helper"));
        assert_eq!(token_at_offset(src, 5), Some("helper"));
        assert_eq!(token_at_offset(src, 10), Some("x"));
        assert!(token_at_offset(src, 9).is_none()); // '('
    }

    #[test]
    fn test_line_col_roundtrip() {
        let src = "fn helper(x: Int) -> String { x }";
        for offset in 0..src.len() {
            if let Some((line, col)) = line_col_from_offset(src, offset) {
                assert_eq!(
                    offset_from_line_col(src, line, col),
                    Some(offset),
                    "roundtrip failed at offset {offset}"
                );
            }
        }
    }
}

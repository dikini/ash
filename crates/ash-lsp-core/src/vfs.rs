//! Virtual File System for tracking open documents in the LSP server.

use dashmap::DashMap;
use lsp_types::TextDocumentContentChangeEvent;
use lsp_types::Uri;
use std::sync::Arc;
use tracing::{debug, info};

/// A single file tracked by the VFS.
#[derive(Debug, Clone)]
pub struct VfsEntry {
    /// The document URI.
    pub uri: Uri,
    /// Document version (monotonically increasing).
    pub version: i32,
    /// Full text content of the document.
    pub content: String,
    /// Byte offsets of the start of each line (including offset 0 for line 0).
    pub line_starts: Vec<usize>,
}

/// Computes line-start byte offsets for the given content.
///
/// Returns a `Vec<usize>` where `line_starts[i]` is the byte offset of the
/// first character on line `i` (0-indexed).  The first element is always `0`.
fn compute_line_starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, ch) in content.char_indices() {
        if ch == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Virtual File System backed by a concurrent `DashMap`.
///
/// Supports LSP open / change / close lifecycle as well as line↔offset
/// conversion helpers.
#[derive(Debug, Default)]
pub struct Vfs {
    inner: DashMap<Uri, VfsEntry>,
}

impl Vfs {
    /// Creates a new, empty VFS.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    /// Inserts or replaces a document in the VFS.
    ///
    /// Computes `line_starts` from the provided `content`.
    pub fn open(&self, uri: Uri, version: i32, content: String) {
        info!(uri = uri.as_str(), version, "VFS open");
        let line_starts = compute_line_starts(&content);
        let entry = VfsEntry {
            uri: uri.clone(),
            version,
            content,
            line_starts,
        };
        self.inner.insert(uri, entry);
    }

    /// Applies incremental or full changes to an open document.
    ///
    /// Follows the LSP `TextDocumentContentChangeEvent` specification:
    /// - If `range` is `None`, the entire document is replaced.
    /// - Otherwise, the text within `range` is replaced by `text`.
    pub fn change(&self, uri: &Uri, version: i32, changes: Vec<TextDocumentContentChangeEvent>) {
        info!(
            uri = uri.as_str(),
            version,
            num_changes = changes.len(),
            "VFS change"
        );
        let mut entry = if let Some(e) = self.inner.get_mut(uri) {
            e
        } else {
            debug!(uri = uri.as_str(), "change called for unknown document");
            return;
        };
        entry.version = version;
        for change in changes {
            match change.range {
                None => {
                    // Full replacement
                    entry.content = change.text;
                    entry.line_starts = compute_line_starts(&entry.content);
                }
                Some(range) => {
                    apply_incremental_change(&mut entry, range, &change.text);
                }
            }
        }
    }

    /// Removes a document from the VFS.
    pub fn close(&self, uri: &Uri) {
        info!(uri = uri.as_str(), "VFS close");
        self.inner.remove(uri);
    }

    /// Returns a snapshot of the VFS entry for `uri`, if it exists.
    pub fn get(&self, uri: &Uri) -> Option<Arc<VfsEntry>> {
        self.inner.get(uri).map(|r| Arc::new(r.value().clone()))
    }

    /// Converts a (line, column) pair to a byte offset within the document.
    ///
    /// Both `line` and `col` are **0-indexed**, matching LSP conventions.
    pub fn line_col_to_offset(&self, uri: &Uri, line: u32, col: u32) -> Option<usize> {
        let entry = self.inner.get(uri)?;
        let line_idx = line as usize;
        let line_starts = &entry.line_starts;
        if line_idx >= line_starts.len() {
            return None;
        }
        let line_start = line_starts[line_idx];
        // Determine the line end: either the next line start minus one, or the
        // end of the content.
        let line_end = if line_idx + 1 < line_starts.len() {
            line_starts[line_idx + 1].saturating_sub(1)
        } else {
            entry.content.len()
        };
        let offset = line_start + col as usize;
        if offset > line_end {
            None
        } else {
            Some(offset)
        }
    }

    /// Converts a byte offset to a (line, column) pair within the document.
    ///
    /// Both `line` and column in the returned tuple are **0-indexed**.
    pub fn offset_to_line_col(&self, uri: &Uri, offset: usize) -> Option<(u32, u32)> {
        let entry = self.inner.get(uri)?;
        let line_starts = &entry.line_starts;
        if offset > entry.content.len() {
            return None;
        }
        // Binary search for the last line_start <= offset.
        let line: usize = match line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let col = offset - line_starts[line];
        Some((line as u32, col as u32))
    }
}

/// Applies a single incremental (range-based) change to a VFS entry.
fn apply_incremental_change(entry: &mut VfsEntry, range: lsp_types::Range, new_text: &str) {
    let line_starts = &entry.line_starts;
    let content = &entry.content;

    // Compute byte offsets from LSP Range (0-indexed lines/cols).
    let start_offset = line_col_to_offset_inner(
        line_starts,
        content.len(),
        range.start.line as usize,
        range.start.character as usize,
    );
    let end_offset = line_col_to_offset_inner(
        line_starts,
        content.len(),
        range.end.line as usize,
        range.end.character as usize,
    );

    let mut new_content = String::with_capacity(
        start_offset + new_text.len() + content.len().saturating_sub(end_offset),
    );
    new_content.push_str(&content[..start_offset]);
    new_content.push_str(new_text);
    new_content.push_str(&content[end_offset..]);

    entry.content = new_content;
    entry.line_starts = compute_line_starts(&entry.content);
}

/// Pure helper: converts a (line, col) pair to a byte offset given line-starts.
fn line_col_to_offset_inner(
    line_starts: &[usize],
    content_len: usize,
    line: usize,
    col: usize,
) -> usize {
    if line >= line_starts.len() {
        return content_len;
    }
    let line_start = line_starts[line];
    let line_end = if line + 1 < line_starts.len() {
        line_starts[line + 1].saturating_sub(1)
    } else {
        content_len
    };
    let offset = line_start + col;
    offset.min(line_end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    fn test_uri() -> Uri {
        "file:///test.ash".parse().unwrap()
    }

    #[test]
    fn test_compute_line_starts_empty() {
        let starts = compute_line_starts("");
        assert_eq!(starts, vec![0]);
    }

    #[test]
    fn test_compute_line_starts_single_line() {
        let starts = compute_line_starts("hello");
        assert_eq!(starts, vec![0]);
    }

    #[test]
    fn test_compute_line_starts_multi_line() {
        let starts = compute_line_starts("line1\nline2\nline3");
        assert_eq!(starts, vec![0, 6, 12]);
    }

    #[test]
    fn test_compute_line_starts_trailing_newline() {
        let starts = compute_line_starts("line1\nline2\n");
        assert_eq!(starts, vec![0, 6, 12]);
    }

    #[test]
    fn test_vfs_open_and_get() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "hello world".to_string());

        let entry = vfs.get(&uri).unwrap();
        assert_eq!(entry.version, 1);
        assert_eq!(entry.content, "hello world");
        assert_eq!(entry.line_starts, vec![0]);
    }

    #[test]
    fn test_vfs_close() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "content".to_string());
        assert!(vfs.get(&uri).is_some());
        vfs.close(&uri);
        assert!(vfs.get(&uri).is_none());
    }

    #[test]
    fn test_vfs_line_col_to_offset() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "hello\nworld\n".to_string());

        assert_eq!(vfs.line_col_to_offset(&uri, 0, 0), Some(0));
        assert_eq!(vfs.line_col_to_offset(&uri, 0, 3), Some(3));
        assert_eq!(vfs.line_col_to_offset(&uri, 1, 0), Some(6));
        assert_eq!(vfs.line_col_to_offset(&uri, 1, 2), Some(8));
        // Beyond content
        assert_eq!(vfs.line_col_to_offset(&uri, 5, 0), None);
    }

    #[test]
    fn test_vfs_offset_to_line_col() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "hello\nworld\n".to_string());

        assert_eq!(vfs.offset_to_line_col(&uri, 0), Some((0, 0)));
        assert_eq!(vfs.offset_to_line_col(&uri, 3), Some((0, 3)));
        assert_eq!(vfs.offset_to_line_col(&uri, 6), Some((1, 0)));
        assert_eq!(vfs.offset_to_line_col(&uri, 8), Some((1, 2)));
        // One past the end is fine (for cursor at EOF)
        assert_eq!(vfs.offset_to_line_col(&uri, 12), Some((2, 0)));
        assert_eq!(vfs.offset_to_line_col(&uri, 13), None);
    }

    #[test]
    fn test_vfs_change_full_replacement() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "old content".to_string());

        vfs.change(
            &uri,
            2,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "new content".to_string(),
            }],
        );

        let entry = vfs.get(&uri).unwrap();
        assert_eq!(entry.version, 2);
        assert_eq!(entry.content, "new content");
    }

    #[test]
    fn test_vfs_change_incremental() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "hello world".to_string());

        // Replace "world" with "rust" → "hello rust"
        vfs.change(
            &uri,
            2,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 6,
                    },
                    end: Position {
                        line: 0,
                        character: 11,
                    },
                }),
                range_length: None,
                text: "rust".to_string(),
            }],
        );

        let entry = vfs.get(&uri).unwrap();
        assert_eq!(entry.content, "hello rust");
    }

    #[test]
    fn test_vfs_change_incremental_insert() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "ab".to_string());

        // Insert "XY" between 'a' and 'b'
        vfs.change(
            &uri,
            2,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 1,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                }),
                range_length: None,
                text: "XY".to_string(),
            }],
        );

        let entry = vfs.get(&uri).unwrap();
        assert_eq!(entry.content, "aXYb");
    }

    #[test]
    fn test_vfs_change_incremental_delete() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "abcde".to_string());

        // Delete "bcd" → "ae"
        vfs.change(
            &uri,
            2,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 1,
                    },
                    end: Position {
                        line: 0,
                        character: 4,
                    },
                }),
                range_length: None,
                text: String::new(),
            }],
        );

        let entry = vfs.get(&uri).unwrap();
        assert_eq!(entry.content, "ae");
    }

    #[test]
    fn test_vfs_change_incremental_multiline() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "line1\nline2\nline3".to_string());

        // Replace "line2\n" with "new2\n"
        vfs.change(
            &uri,
            2,
            vec![TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 1,
                        character: 0,
                    },
                    end: Position {
                        line: 2,
                        character: 0,
                    },
                }),
                range_length: None,
                text: "new2\n".to_string(),
            }],
        );

        let entry = vfs.get(&uri).unwrap();
        assert_eq!(entry.content, "line1\nnew2\nline3");
        assert_eq!(entry.line_starts, vec![0, 6, 11]);
    }

    #[test]
    fn test_vfs_change_unknown_uri() {
        let vfs = Vfs::new();
        let uri = test_uri();
        // change on unknown URI should be a no-op
        vfs.change(
            &uri,
            1,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "content".to_string(),
            }],
        );
        assert!(vfs.get(&uri).is_none());
    }

    #[test]
    fn test_vfs_multiple_incremental_changes() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "abc".to_string());

        vfs.change(
            &uri,
            2,
            vec![
                TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 0,
                            character: 1,
                        },
                        end: Position {
                            line: 0,
                            character: 2,
                        },
                    }),
                    range_length: None,
                    text: "X".to_string(),
                },
                TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position {
                            line: 0,
                            character: 2,
                        },
                        end: Position {
                            line: 0,
                            character: 3,
                        },
                    }),
                    range_length: None,
                    text: "Y".to_string(),
                },
            ],
        );

        let entry = vfs.get(&uri).unwrap();
        assert_eq!(entry.content, "aXY");
    }
}

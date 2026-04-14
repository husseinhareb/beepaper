//! Helpers for maintaining wallpaper selection history.

use std::path::PathBuf;

/// Append a selection to history and trim it to the configured maximum size.
pub fn push_selection(history: &mut Vec<PathBuf>, selection: PathBuf, max_size: usize) {
    history.push(selection);
    trim(history, max_size);
}

/// Trim history in place, keeping only the newest `max_size` entries.
pub fn trim(history: &mut Vec<PathBuf>, max_size: usize) {
    if max_size == 0 {
        history.clear();
        return;
    }

    let overflow = history.len().saturating_sub(max_size);
    if overflow > 0 {
        history.drain(0..overflow);
    }
}

/// Return recent history entries from newest to oldest.
pub fn recent(history: &[PathBuf], limit: Option<usize>) -> Vec<PathBuf> {
    match limit {
        Some(limit) => history.iter().rev().take(limit).cloned().collect(),
        None => history.iter().rev().cloned().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{push_selection, recent, trim};
    use std::path::PathBuf;

    #[test]
    fn trim_keeps_newest_entries() {
        let mut history = vec![
            PathBuf::from("one.jpg"),
            PathBuf::from("two.jpg"),
            PathBuf::from("three.jpg"),
        ];

        trim(&mut history, 2);

        assert_eq!(
            history,
            vec![PathBuf::from("two.jpg"), PathBuf::from("three.jpg")]
        );
    }

    #[test]
    fn push_selection_trims_to_maximum_size() {
        let mut history = vec![PathBuf::from("one.jpg"), PathBuf::from("two.jpg")];

        push_selection(&mut history, PathBuf::from("three.jpg"), 2);

        assert_eq!(
            history,
            vec![PathBuf::from("two.jpg"), PathBuf::from("three.jpg")]
        );
    }

    #[test]
    fn recent_returns_newest_first() {
        let history = vec![
            PathBuf::from("one.jpg"),
            PathBuf::from("two.jpg"),
            PathBuf::from("three.jpg"),
        ];

        assert_eq!(
            recent(&history, Some(2)),
            vec![PathBuf::from("three.jpg"), PathBuf::from("two.jpg")]
        );
    }
}

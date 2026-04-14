//! Random wallpaper selection helpers.

use std::collections::HashSet;
use std::path::PathBuf;

use rand::Rng;
use rand::seq::SliceRandom;

use crate::error::SelectionError;

/// Select a random wallpaper while avoiding recently used entries when possible.
pub fn select_random<R: Rng + ?Sized>(
    candidates: &[PathBuf],
    history: &[PathBuf],
    no_repeat_window: usize,
    rng: &mut R,
) -> Result<PathBuf, SelectionError> {
    if candidates.is_empty() {
        return Err(SelectionError::NoCandidates);
    }

    let recent: HashSet<&PathBuf> = history.iter().rev().take(no_repeat_window).collect();
    let eligible: Vec<&PathBuf> = candidates
        .iter()
        .filter(|candidate| !recent.contains(candidate))
        .collect();

    let pool = if eligible.is_empty() {
        candidates.iter().collect::<Vec<_>>()
    } else {
        eligible
    };

    pool.choose(rng)
        .map(|selected| (*selected).clone())
        .ok_or(SelectionError::NoCandidates)
}

/// Select a random wallpaper using the thread-local RNG.
pub fn select_random_thread_rng(
    candidates: &[PathBuf],
    history: &[PathBuf],
    no_repeat_window: usize,
) -> Result<PathBuf, SelectionError> {
    let mut rng = rand::thread_rng();
    select_random(candidates, history, no_repeat_window, &mut rng)
}

#[cfg(test)]
mod tests {
    use super::select_random;
    use anyhow::Result;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::path::PathBuf;

    #[test]
    fn selection_avoids_recent_items_when_possible() -> Result<()> {
        let candidates = vec![
            PathBuf::from("one.jpg"),
            PathBuf::from("two.jpg"),
            PathBuf::from("three.jpg"),
        ];
        let history = vec![PathBuf::from("one.jpg"), PathBuf::from("two.jpg")];
        let mut rng = StdRng::seed_from_u64(42);

        let selected = select_random(&candidates, &history, 2, &mut rng)?;

        assert_eq!(selected, PathBuf::from("three.jpg"));
        Ok(())
    }

    #[test]
    fn selection_falls_back_when_everything_is_recent() -> Result<()> {
        let candidates = vec![PathBuf::from("one.jpg")];
        let history = vec![PathBuf::from("one.jpg")];
        let mut rng = StdRng::seed_from_u64(42);

        let selected = select_random(&candidates, &history, 1, &mut rng)?;

        assert_eq!(selected, PathBuf::from("one.jpg"));
        Ok(())
    }
}

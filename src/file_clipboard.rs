//! Process-local cut intent, tied to the system pasteboard revision.
//! No file bytes, timers or persistent state are needed for marking a cut.
use std::path::PathBuf;

#[derive(Default)]
pub struct CutClipboard {
    batch: Option<CutBatch>,
}
struct CutBatch {
    revision: isize,
    paths: Vec<PathBuf>,
    busy: bool,
}
impl CutClipboard {
    pub fn set(&mut self, revision: isize, mut paths: Vec<PathBuf>) {
        paths.sort();
        paths.dedup();
        self.batch = (!paths.is_empty()).then_some(CutBatch {
            revision,
            paths,
            busy: false,
        });
    }
    pub fn clear(&mut self) {
        self.batch = None;
    }
    pub fn paths(&mut self, revision: isize) -> Option<&[PathBuf]> {
        if self.batch.as_ref().is_some_and(|b| b.revision != revision) {
            self.clear();
        }
        self.batch.as_ref().map(|b| b.paths.as_slice())
    }
    pub fn begin(&mut self, revision: isize) -> bool {
        if self.paths(revision).is_none() {
            return false;
        }
        let batch = self.batch.as_mut().unwrap();
        if batch.busy {
            return false;
        }
        batch.busy = true;
        true
    }
    /// Remove only successful moves; conflicts, failures and cancellations can retry.
    /// An old job must never alter a newer cut batch.
    pub fn complete(&mut self, revision: isize, moved: &[PathBuf]) -> Option<Vec<PathBuf>> {
        let batch = self.batch.as_mut()?;
        if batch.revision != revision || !batch.busy {
            return None;
        }
        batch.paths.retain(|path| !moved.contains(path));
        batch.busy = false;
        let remaining = batch.paths.clone();
        if remaining.is_empty() {
            self.clear();
        }
        Some(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipboard_replacement_and_stale_completion_never_move_new_contents() {
        let old = PathBuf::from("/资料/报告 😀.txt");
        let new = PathBuf::from("/资料/新的文件夹");
        let mut cut = CutClipboard::default();
        cut.set(10, vec![old.clone()]);
        assert!(cut.begin(10));
        cut.set(11, vec![new.clone()]);
        assert!(cut.complete(10, &[old]).is_none());
        assert_eq!(cut.paths(11).unwrap(), &[new]);
        assert!(cut.paths(12).is_none()); // Another app copied even identical URLs.
        assert!(!cut.begin(12));
        cut.set(13, vec![PathBuf::from("/资料/a")]);
        cut.clear(); // Explicit copy cancels a pending cut.
        assert!(cut.paths(13).is_none());
    }
    #[test]
    fn partial_moves_retry_only_remaining_items_and_block_duplicate_pastes() {
        let a = PathBuf::from("/资料/a.txt");
        let b = PathBuf::from("/资料/中文文件夹");
        let mut cut = CutClipboard::default();
        cut.set(20, vec![b.clone(), a.clone(), b.clone()]);
        assert_eq!(cut.paths(20).unwrap().len(), 2);
        assert!(cut.begin(20));
        assert!(!cut.begin(20));
        assert_eq!(cut.complete(20, &[a]).unwrap(), vec![b.clone()]);
        assert!(cut.begin(20));
        assert_eq!(cut.complete(20, &[]).unwrap(), vec![b.clone()]);
        assert!(cut.begin(20));
        assert!(cut.complete(20, &[b]).unwrap().is_empty());
        assert!(cut.paths(20).is_none());
    }
}

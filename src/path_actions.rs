//! Path shortcuts use lexical paths and pass terminal arguments without a shell.
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};

pub fn browsing_directory(root: &Path, chain: &[PathBuf], column: Option<usize>) -> PathBuf {
    column
        .and_then(|k| k.checked_sub(1))
        .and_then(|k| chain.get(k))
        .map_or_else(|| root.to_path_buf(), Clone::clone)
}

/// Full paths retain the item name. Folder paths strip the final component,
/// including for a selected folder; an empty selection uses the browsing dir.
pub fn clipboard_text(selected: &[PathBuf], current: &Path, folder_only: bool) -> String {
    if selected.is_empty() {
        return current.to_string_lossy().into_owned();
    }
    let mut selected = selected.to_vec();
    selected.sort();
    let mut seen = HashSet::new();
    selected
        .iter()
        .filter_map(|path| {
            let path = if folder_only {
                path.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(path)
            } else {
                path
            };
            seen.insert(path.to_path_buf())
                .then(|| path.to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn terminal_command(directory: &Path) -> Command {
    let mut command = Command::new("/usr/bin/open");
    command
        .args(["-a", "/System/Applications/Utilities/Terminal.app", "--"])
        .arg(directory);
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_and_folder_paths_keep_unicode_and_deduplicate_parents() {
        let root = Path::new("/资料");
        let selection = vec![
            PathBuf::from("/资料/文件夹/第二份.txt"),
            PathBuf::from("/资料/文件夹/报告 😀.pdf"),
        ];
        assert_eq!(
            clipboard_text(&selection, root, false),
            "/资料/文件夹/报告 😀.pdf\n/资料/文件夹/第二份.txt"
        );
        assert_eq!(clipboard_text(&selection, root, true), "/资料/文件夹");
        assert_eq!(
            clipboard_text(&[PathBuf::from("/资料/文件夹")], root, true),
            "/资料"
        );
        assert_eq!(clipboard_text(&[PathBuf::from("/")], root, true), "/");
        for folder in [false, true] {
            assert_eq!(clipboard_text(&[], root, folder), "/资料");
        }
    }
    #[test]
    fn terminal_uses_active_column_and_preserves_literal_arguments() {
        let root = Path::new("/资料");
        let chain = vec![
            PathBuf::from("/资料/中文 😀 $() `字`"),
            PathBuf::from("/资料/中文 😀 $() `字`/子目录"),
        ];
        assert_eq!(browsing_directory(root, &chain, None), root);
        assert_eq!(browsing_directory(root, &chain, Some(0)), root);
        let dir = browsing_directory(root, &chain, Some(1));
        assert_eq!(dir, chain[0]);
        assert_eq!(browsing_directory(root, &chain, Some(2)), chain[1]);
        assert_eq!(browsing_directory(root, &chain, Some(99)), root);
        let command = terminal_command(&dir);
        assert_eq!(command.get_program(), "/usr/bin/open");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec![
                std::ffi::OsStr::new("-a"),
                std::ffi::OsStr::new("/System/Applications/Utilities/Terminal.app"),
                std::ffi::OsStr::new("--"),
                dir.as_os_str()
            ]
        );
    }
}

use std::fs;
use std::path::{Path, PathBuf};

/// Rapidly resolves the current git branch name without spawning a subprocess whenever possible.
pub fn get_git_branch(start_path: &Path) -> Option<String> {
    let git_dir = find_git_dir(start_path)?;
    get_branch_from_git_dir(&git_dir)
}

fn find_git_dir(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent()?
    } else {
        start_path
    };

    loop {
        let dot_git = current.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        } else if dot_git.is_file()
            && let Ok(content) = fs::read_to_string(&dot_git)
            && let Some(target) = content.trim().strip_prefix("gitdir:")
        {
            let target_path = PathBuf::from(target.trim());
            if target_path.is_absolute() {
                return Some(target_path);
            } else {
                return Some(current.join(target_path));
            }
        }

        match current.parent() {
            Some(p) if p != current => current = p,
            _ => break,
        }
    }

    None
}

fn get_branch_from_git_dir(git_dir: &Path) -> Option<String> {
    let head_path = git_dir.join("HEAD");
    let content = fs::read_to_string(head_path).ok()?;
    let line = content.trim();

    if let Some(ref_path) = line.strip_prefix("ref: refs/heads/") {
        Some(ref_path.to_string())
    } else if line.len() >= 7 {
        // Detached HEAD: return short SHA
        Some(line[..7].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_non_git_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let branch = get_git_branch(temp_dir.path());
        assert!(branch.is_none());
    }

    #[test]
    fn test_git_branch_resolution() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dot_git = temp_dir.path().join(".git");
        fs::create_dir_all(&dot_git).unwrap();

        let mut head = File::create(dot_git.join("HEAD")).unwrap();
        head.write_all(b"ref: refs/heads/feature-awesome\n")
            .unwrap();

        let branch = get_git_branch(temp_dir.path());
        assert_eq!(branch.as_deref(), Some("feature-awesome"));
    }

    #[test]
    fn test_detached_head_resolution() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dot_git = temp_dir.path().join(".git");
        fs::create_dir_all(&dot_git).unwrap();

        let mut head = File::create(dot_git.join("HEAD")).unwrap();
        head.write_all(b"d58460183921abc456\n").unwrap();

        let branch = get_git_branch(temp_dir.path());
        assert_eq!(branch.as_deref(), Some("d584601"));
    }
}

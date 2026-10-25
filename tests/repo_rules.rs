use std::fs;
use std::path::Path;

fn collect_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = fs::read_dir(dir).expect("source directory must be readable");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Parallel test and `cargo-mutants` workers share the system temporary
/// directory, so tests that build paths under `std::env::temp_dir()` can
/// race across workers or threads. Tests must use `tempfile::tempdir()`
/// instead.
#[test]
fn tests_use_isolated_tempdirs_instead_of_shared_temp_dir() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_rs_files(&root.join("src"), &mut files);
    collect_rs_files(&root.join("tests"), &mut files);

    for path in files {
        if path.file_name().and_then(|n| n.to_str()) == Some("repo_rules.rs") {
            continue;
        }
        let content = fs::read_to_string(&path).expect("Rust file must be readable");
        let test_section = match content.find("#[cfg(test)]") {
            Some(idx) => &content[idx..],
            None if path.starts_with(root.join("tests")) => &content[..],
            None => continue,
        };

        assert!(
            !test_section.contains("temp_dir()"),
            "{} uses `temp_dir()` in test code; use `tempfile::tempdir()` so parallel workers stay isolated",
            path.strip_prefix(root).unwrap_or(&path).display()
        );
    }
}

/// Every helper must land with its first production caller rather than
/// ahead of time behind `#[allow(dead_code)]`.
#[test]
fn production_code_does_not_allow_dead_code() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_rs_files(&root.join("src"), &mut files);

    for path in files {
        let content = fs::read_to_string(&path).expect("Rust file must be readable");
        let prod_section = match content.find("#[cfg(test)]") {
            Some(idx) => &content[..idx],
            None => &content[..],
        };

        assert!(
            !prod_section.contains("allow(dead_code)"),
            "{} contains `allow(dead_code)` in production code; land helpers with their first caller",
            path.strip_prefix(root).unwrap_or(&path).display()
        );
    }
}

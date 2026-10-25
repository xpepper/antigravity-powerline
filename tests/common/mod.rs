//! Fixture for the binary-level wiring tests: a throwaway `HOME` holding an
//! optional config, and a way to run one status line Refresh against it. Each
//! test crate uses a different subset of it.
#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

pub struct Sandbox {
    home: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            home: tempfile::tempdir().unwrap(),
        }
    }

    pub fn home(&self) -> &Path {
        self.home.path()
    }

    pub fn gemini_dir(&self) -> PathBuf {
        self.home().join(".gemini")
    }

    pub fn state_dir(&self) -> PathBuf {
        let cache = if cfg!(target_os = "macos") {
            self.home().join("Library/Caches")
        } else {
            self.home().join(".cache")
        };
        cache.join("antigravity-powerline")
    }

    pub fn write_config(&self, toml: &str) {
        fs::create_dir_all(self.gemini_dir()).unwrap();
        fs::write(self.gemini_dir().join("powerline.toml"), toml).unwrap();
    }

    fn command(&self, args: &[&str], envs: &[(&str, &Path)]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_antigravity-powerline"));
        command
            .args(args)
            .envs(envs.iter().copied())
            .env("HOME", self.home())
            .env_remove("XDG_CACHE_HOME")
            .current_dir(self.home());
        command
    }

    /// Runs one Refresh and returns its stdout without the trailing newline.
    ///
    /// `HOME` and the working directory are set on the child only, so tests
    /// can run in parallel without sharing process state.
    pub fn refresh(&self, payload: &str, args: &[&str]) -> String {
        self.refresh_with_env(payload, args, &[])
    }

    /// Like `refresh`, with extra environment variables set on the child.
    pub fn refresh_with_env(&self, payload: &str, args: &[&str], envs: &[(&str, &Path)]) -> String {
        let mut child = self
            .command(args, envs)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "refresh failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .trim_end()
            .to_string()
    }

    /// State files whose name starts with `prefix`, e.g. `pr_`.
    pub fn state_files(&self, prefix: &str) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(self.state_dir()) else {
            return Vec::new();
        };
        entries
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(prefix))
            })
            .collect()
    }
}

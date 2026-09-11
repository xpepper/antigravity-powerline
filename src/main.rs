use clap::Parser;
use std::io::{self, Read};
use std::path::Path;

mod cli;
mod config;
mod git;
mod github;
mod icons;
mod input;
mod renderer;
mod segments;
mod theme;

use cli::Cli;
use config::{Config, IconSet, Style};
use input::AntigravityInput;
use renderer::render_segments;
use segments::artifacts::render_artifacts_segment;
use segments::cache::render_cache_segment;
use segments::git::render_git_segment;
use segments::model::render_model_segment;
use segments::pr::render_pr_segment;
use segments::quota::render_quota_segment;
use segments::tokens::render_tokens_segment;
use segments::total_tokens::render_total_tokens_segment;
use theme::Palette;

fn read_stdin() -> String {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let _ = handle.read_to_string(&mut buffer);
    buffer
}

fn main() {
    let cli = Cli::parse();

    if let Some(cache_path) = cli.fetch_pr_cache {
        let repo_dir = cli
            .repo_dir
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        github::fetch_and_write_pr_cache(&repo_dir, &cache_path);
        return;
    }

    if cli.init {
        let default_cfg = Config::default();
        let toml_str = default_cfg
            .to_toml_string()
            .unwrap_or_else(|_| String::new());

        if let Some(cfg_path) = Config::default_config_path() {
            if cfg_path.exists() {
                eprintln!("Config file already exists at {}", cfg_path.display());
            } else {
                if let Some(parent) = cfg_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(&cfg_path, &toml_str) {
                    Ok(_) => println!("Initialized config at {}", cfg_path.display()),
                    Err(e) => eprintln!("Failed to write config: {e}"),
                }
            }
        } else {
            println!("{toml_str}");
        }
        return;
    }

    let mut config = Config::load_from_file_or_default(cli.config.as_deref());

    if let Some(s) = cli.style.and_then(|s| s.parse::<Style>().ok()) {
        config.style = s;
    }

    if let Some(i) = cli.icon_set.and_then(|i| i.parse::<IconSet>().ok()) {
        config.icon_set = i;
    }

    if let Some(theme_override) = cli.theme {
        config.theme = theme_override;
    }

    let palette = Palette::for_theme(&config.theme);

    let raw_input = read_stdin();
    let input = if raw_input.trim().is_empty() {
        AntigravityInput::default()
    } else {
        AntigravityInput::from_json(&raw_input)
    };

    let git_branch = input
        .resolved_cwd()
        .and_then(|cwd| git::get_git_branch(Path::new(cwd)));

    let gh_available = github::is_gh_available();
    let pr_info = if config.pr.enabled
        && config.segments.iter().any(|s| s == "pr")
        && let (Some(cwd), Some(branch)) = (input.resolved_cwd(), git_branch.as_deref())
    {
        github::get_pr_info(
            Path::new(cwd),
            branch,
            config.pr.cache_ttl_seconds,
            gh_available,
        )
    } else {
        None
    };

    let mut rendered_segments = Vec::new();

    for seg in &config.segments {
        match seg.as_str() {
            "model" => {
                if let Some(s) = render_model_segment(
                    input.model.as_ref(),
                    &config.model,
                    config.icon_set,
                    &palette,
                ) {
                    rendered_segments.push(s);
                }
            }
            "git" => {
                if let Some(s) = render_git_segment(
                    git_branch.as_deref(),
                    &config.git,
                    config.icon_set,
                    &palette,
                ) {
                    rendered_segments.push(s);
                }
            }
            "pr" => {
                if let Some(s) =
                    render_pr_segment(pr_info.as_ref(), &config.pr, config.icon_set, &palette)
                {
                    rendered_segments.push(s);
                }
            }
            "tokens" => {
                if let Some(s) = render_tokens_segment(
                    &input.context_window,
                    &config.tokens,
                    config.icon_set,
                    &palette,
                ) {
                    rendered_segments.push(s);
                }
            }
            "quota" => {
                if let Some(s) =
                    render_quota_segment(&input.quota, &config.quota, config.icon_set, &palette)
                {
                    rendered_segments.push(s);
                }
            }
            "cache" => {
                if let Some(s) = render_cache_segment(
                    &input.context_window,
                    &config.cache,
                    config.icon_set,
                    &palette,
                ) {
                    rendered_segments.push(s);
                }
            }
            "total_tokens" => {
                if let Some(s) = render_total_tokens_segment(
                    &input.context_window,
                    &config.total_tokens,
                    config.icon_set,
                    &palette,
                ) {
                    rendered_segments.push(s);
                }
            }
            "artifacts" => {
                if let Some(s) = render_artifacts_segment(
                    input.artifact_count,
                    &config.artifacts,
                    config.icon_set,
                    &palette,
                ) {
                    rendered_segments.push(s);
                }
            }
            _ => {}
        }
    }

    let output = render_segments(&rendered_segments, config.style, &palette);
    println!("{output}");
}

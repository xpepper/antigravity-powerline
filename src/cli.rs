use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "antigravity-powerline")]
#[command(author, version, about = "Customizable status line for Google Antigravity CLI", long_about = None)]
pub struct Cli {
    /// Path to TOML configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override the display style (minimal, powerline, capsule, plain)
    #[arg(short, long)]
    pub style: Option<String>,

    /// Override the color theme (colorblind, github, nord, tokyo-night, plain)
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Override the icon set (plain, nerd, emoji)
    #[arg(short = 'i', long = "icon-set")]
    pub icon_set: Option<String>,

    /// Generate a default powerline.toml configuration file
    #[arg(long)]
    pub init: bool,
}

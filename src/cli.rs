use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Shortcut API CLI – bulk create Objectives, Epics, and Stories.
#[derive(Parser, Debug)]
#[command(name = "bypass", version, about)]
pub struct Cli {
    /// Shortcut API token [env: SHORTCUT_API_TOKEN]
    #[arg(long, env = "SHORTCUT_API_TOKEN", global = true, hide_env_values = true)]
    pub token: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create Shortcut resources from an input file (.yaml, .csv, .xlsx).
    Create(CreateArgs),
}

#[derive(clap::Args, Debug)]
pub struct CreateArgs {
    /// Input file (.yaml/.yml, .csv, or .xlsx).
    /// YAML files may contain objectives, epics, and stories in a single file.
    /// CSV/XLSX files require --type to specify which resource kind to import.
    #[arg(long, short, value_name = "FILE")]
    pub file: PathBuf,

    /// Resource type – required for CSV and XLSX files.
    /// YAML files determine the type from top-level keys (objectives/epics/stories).
    #[arg(long, value_enum, value_name = "TYPE")]
    pub r#type: Option<ResourceType>,

    /// Markdown template file whose rendered content becomes the description for
    /// every epic that does not supply its own inline template.
    /// Template variables: {{name}}, {{description}}, {{objective}},
    ///   {{owners}}, {{teams}}, {{labels}}, {{start_date}}, {{deadline}}.
    #[arg(long, value_name = "FILE")]
    pub template: Option<PathBuf>,

    /// Validate names and structure without creating any resources.
    /// Still contacts the API to resolve member/group/workflow names.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub output: OutputFormat,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ResourceType {
    Objective,
    Epic,
    Story,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable coloured output (default).
    Text,
    /// Newline-delimited JSON records – one per created resource or error.
    Json,
}

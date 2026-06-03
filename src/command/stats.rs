use std::collections::BTreeMap;

use clap::Parser;
use serde::Serialize;
use walkdir::WalkDir;

use crate::{
    utils::{
        error::CliResult,
        output::{OutputConfig, emit_json_data},
    },
};

const STATS_EXAMPLES: &str = "\
EXAMPLES:
    libra stats                  Count file extensions in the current directory
    libra --json stats           Structured JSON output";

#[derive(Parser, Debug)]
#[command(
    about = "Count file extensions in the current working directory",
    after_help = STATS_EXAMPLES
)]
pub struct StatsArgs;

#[derive(Debug, Clone, Serialize)]
pub struct StatsOutput {
    pub extensions: BTreeMap<String, usize>,
}

fn run_stats() -> CliResult<StatsOutput> {
    let mut ext_map = BTreeMap::new();

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| {
            e.path().components().all(|c| {
                let name = c.as_os_str().to_string_lossy();
                name != ".libra" && name != "target"
            })
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let ext = entry
            .path()
            .extension()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "no_extension".to_string());
        *ext_map.entry(ext).or_insert(0) += 1;
    }

    Ok(StatsOutput { extensions: ext_map })
}

fn render_stats_output(output: &StatsOutput, writer: &mut impl std::io::Write) -> CliResult<()> {
    for (ext, count) in &output.extensions {
        writeln!(writer, "{:<20} {}", ext, count)?;
    }
    Ok(())
}

pub async fn execute_safe(_args: StatsArgs, output: &OutputConfig) -> CliResult<()> {
    let stats = run_stats()?;

    if output.is_json() {
        emit_json_data("stats", &stats, output)?;
    } else if !output.quiet {
        let mut stdout = std::io::stdout();
        render_stats_output(&stats, &mut stdout)?;
    }

    Ok(())
}

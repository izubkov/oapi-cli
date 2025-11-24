mod filter;
mod io;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oapi-cli", version, about = "OpenAPI command-line toolkit")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Filter paths and prune unused components
    Filter {
        /// Input OpenAPI file (JSON or YAML)
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Output OpenAPI file (JSON or YAML). Extension determines format (.json/.yaml/.yml)
        #[arg(short = 'o', long = "output")]
        output: PathBuf,

        /// Path(s) or prefix(es) to keep. Can be passed multiple times.
        /// Use '*' suffix for prefix matching (e.g., '/api/v1/*')
        #[arg(long = "path")]
        paths: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Filter {
            input,
            output,
            paths,
        } => {
            if paths.is_empty() {
                eprintln!("Error: at least one --path must be provided");
                std::process::exit(2);
            }
            let mut spec = io::load_spec(&input)?;
            filter::filter_paths(&mut spec, &paths);
            let refs = filter::collect_refs(&spec);
            filter::prune_components(&mut spec, &refs);
            io::save_spec(&spec, &output)?;
            println!("Filtered spec written to {}", output.display());
        }
    }

    Ok(())
}

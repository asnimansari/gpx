//! Command-line interface for the gpx-rs tool.

mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "gpx",
    about = "A command-line tool for working with GPX files",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a GPX file against the GPX 1.1 schema
    Validate {
        /// Path to the input GPX file
        input_file: PathBuf,
        /// Treat warnings as failures (non-zero exit code)
        #[arg(long)]
        strict: bool,
        /// Output the validation report in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Show information and statistics about a GPX file
    Info {
        /// Path to the input GPX file
        input_file: PathBuf,
        /// Output information in JSON format
        #[arg(long)]
        json: bool,
        /// Validate the input against the GPX 1.1 schema first
        #[arg(long)]
        strict: bool,
    },
    /// Edit a GPX file with various transformations
    Edit {
        /// Path to the input GPX file
        input_file: PathBuf,
        /// Path to the output file
        #[arg(short, long)]
        output_file: PathBuf,
        /// Validate the input against the GPX 1.1 schema first
        #[arg(long)]
        strict: bool,
        #[command(flatten)]
        options: Box<commands::EditOptions>,
    },
    /// Merge multiple GPX files into one
    Merge {
        /// Input GPX files to merge
        input_files: Vec<PathBuf>,
        /// Path to the output file
        #[arg(short, long)]
        output_file: PathBuf,
        /// Validate each input against the GPX 1.1 schema first
        #[arg(long)]
        strict: bool,
    },
    /// Convert between GPX, GeoJSON, and KML formats
    Convert {
        /// Path to the input file
        input_file: PathBuf,
        /// Path to the output file
        #[arg(short, long)]
        output_file: PathBuf,
        /// Validate GPX input against the GPX 1.1 schema first
        #[arg(long)]
        strict: bool,
    },
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Validate {
            input_file,
            strict,
            json,
        } => commands::validate(&input_file, strict, json),
        Command::Info {
            input_file,
            json,
            strict,
        } => commands::info(&input_file, json, strict),
        Command::Edit {
            input_file,
            output_file,
            strict,
            options,
        } => commands::edit(&input_file, &output_file, strict, *options),
        Command::Merge {
            input_files,
            output_file,
            strict,
        } => commands::merge(&input_files, &output_file, strict),
        Command::Convert {
            input_file,
            output_file,
            strict,
        } => commands::convert(&input_file, &output_file, strict),
    };

    match result {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::from(1)
        }
    }
}

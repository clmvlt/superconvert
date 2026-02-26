mod notify;
mod platform;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use superconvert_engine::{
    ConversionOptions, OutputFormat, all_supported_extensions, convert_single_file,
    output_formats_for_extension,
};

#[derive(Parser)]
#[command(name = "superconvert-cli", version, about = "SuperConvert CLI - file converter and context menu installer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a file to another format
    Convert {
        /// Input file path
        file: PathBuf,
        /// Target format (e.g. jpg, png, mp3, pdf)
        #[arg(long = "to")]
        to: String,
        /// Quality (0-100, for lossy formats)
        #[arg(long, value_parser = clap::value_parser!(u8))]
        quality: Option<u8>,
    },
    /// List available output formats
    ListFormats {
        /// Only show formats for this input extension
        #[arg(long)]
        input: Option<String>,
    },
    /// Install or uninstall OS context menu integration
    ContextMenu {
        #[command(subcommand)]
        action: ContextMenuAction,
    },
}

#[derive(Subcommand)]
enum ContextMenuAction {
    /// Register context menu entries in the OS
    Install,
    /// Remove context menu entries from the OS
    Uninstall,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Convert { file, to, quality } => cmd_convert(&file, &to, quality),
        Commands::ListFormats { input } => cmd_list_formats(input.as_deref()),
        Commands::ContextMenu { action } => match action {
            ContextMenuAction::Install => platform::install_context_menu(),
            ContextMenuAction::Uninstall => platform::uninstall_context_menu(),
        },
    };

    if let Err(e) = result {
        notify::show_notification("SuperConvert", &format!("Error: {}", e), true);
        std::process::exit(1);
    }
}

fn cmd_convert(file: &Path, to: &str, quality: Option<u8>) -> Result<()> {
    if !file.exists() {
        bail!("File not found: {}", file.display());
    }

    let output_format = OutputFormat::from_extension(to)
        .with_context(|| format!("Unknown output format: {}", to))?;

    // Build output path: same folder, same stem, new extension
    let parent = file.parent().unwrap_or(Path::new("."));
    let stem = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let out_ext = output_format_file_extension(&output_format);
    let mut output_path = parent.join(format!("{}.{}", stem, out_ext));

    // Avoid overwriting: add _1, _2, etc.
    let mut counter = 1u32;
    while output_path.exists() {
        output_path = parent.join(format!("{}_{}.{}", stem, counter, out_ext));
        counter += 1;
    }

    let options = ConversionOptions {
        output_format,
        quality,
    };

    convert_single_file(file, &output_path, &options)
        .with_context(|| format!("Conversion failed for {}", file.display()))?;

    let msg = format!(
        "Converted {} to {}",
        file.file_name().unwrap_or_default().to_string_lossy(),
        output_path.file_name().unwrap_or_default().to_string_lossy()
    );
    notify::show_notification("SuperConvert", &msg, false);

    Ok(())
}

fn cmd_list_formats(input_ext: Option<&str>) -> Result<()> {
    match input_ext {
        Some(ext) => {
            let formats = output_formats_for_extension(ext);
            if formats.is_empty() {
                println!("No output formats available for .{}", ext);
            } else {
                println!("Output formats for .{}:", ext);
                for f in formats {
                    let quality_note = if f.supports_quality { " (quality)" } else { "" };
                    println!("  {} - [{}]{}", f.label, f.category, quality_note);
                }
            }
        }
        None => {
            println!("Supported input extensions:");
            let exts = all_supported_extensions();
            for chunk in exts.chunks(10) {
                println!("  {}", chunk.join(", "));
            }
        }
    }
    Ok(())
}

/// Get the file extension string for an OutputFormat (handles archive compound extensions).
fn output_format_file_extension(fmt: &OutputFormat) -> String {
    match fmt {
        OutputFormat::Archive(af) => af.extension().to_string(),
        _ => fmt.extension().to_string(),
    }
}

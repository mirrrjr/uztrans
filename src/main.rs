mod cli;
mod config;
mod diff;
mod error;
mod html;
mod io;
mod markdown;
mod processor;
mod transliterator;
mod walker;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use rayon::prelude::*;

use cli::Cli;
use config::ExtensionConfig;
use walker::Filters;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("uztrans: {e}");
        return ExitCode::from(2);
    }

    match run(&cli) {
        Ok(had_failures) => {
            if had_failures {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("uztrans: {e}");
            ExitCode::from(2)
        }
    }
}

/// Runs the requested operation. Returns `Ok(true)` if the run as a whole
/// succeeded but *some individual files* were skipped due to recoverable
/// errors (binary content, permission issues, etc.) — that's a
/// partial-failure exit code, not a hard error.
fn run(cli: &Cli) -> error::Result<bool> {
    if cli.paths.is_empty() {
        return run_stdin(cli);
    }
    run_paths(cli)
}

fn run_stdin(cli: &Cli) -> error::Result<bool> {
    let source = io::read_stdin_to_string()?;
    let result = processor::process(&source, config::DocKind::PlainText);

    if cli.diff {
        diff::print_diff("<stdin>", &source, &result.output);
    }
    if !cli.dry_run && !cli.diff {
        match &cli.output {
            Some(path) => io::write_string(path, &result.output)?,
            None => io::write_stdout(&result.output)?,
        }
    }
    Ok(false)
}

fn run_paths(cli: &Cli) -> error::Result<bool> {
    let ext_config = ExtensionConfig::new(cli.extra_extensions.iter().cloned());
    let filters = Filters::build(&cli.include, &cli.exclude)?;

    let files = walker::collect_files(&cli.paths, cli.recursive, &ext_config, &filters)?;

    if files.is_empty() && !cli.quiet {
        eprintln!("uztrans: no matching files found");
    }

    // Single-file, non-directory invocations support `-o <file>` meaning
    // "write the transformed file here"; directory invocations treat `-o`
    // as an output *root* that mirrors the input tree (by file name, best
    // effort — see mirror_path).
    let single_file_output = if cli.paths.len() == 1 && cli.paths[0].is_file() {
        cli.output.clone()
    } else {
        None
    };
    let output_root = if single_file_output.is_none() {
        cli.output.clone()
    } else {
        None
    };

    let process_one = |path: &PathBuf| -> (PathBuf, error::Result<bool>) {
        let outcome = process_single_file(
            cli,
            path,
            single_file_output.as_deref(),
            output_root.as_deref(),
        );
        (path.clone(), outcome)
    };

    let results: Vec<(PathBuf, error::Result<bool>)> = if cli.parallel && files.len() > 1 {
        files.par_iter().map(process_one).collect()
    } else {
        files.iter().map(process_one).collect()
    };

    let mut had_failures = false;
    for (path, outcome) in results {
        match outcome {
            Ok(changed) => {
                if cli.verbose {
                    let verb = if cli.dry_run {
                        "would change"
                    } else if changed {
                        "changed"
                    } else {
                        "unchanged"
                    };
                    eprintln!("{verb}: {}", path.display());
                }
            }
            Err(e) => {
                had_failures = true;
                if !cli.quiet {
                    eprintln!("uztrans: skipping {}: {e}", path.display());
                }
            }
        }
    }

    Ok(had_failures)
}

fn process_single_file(
    cli: &Cli,
    path: &Path,
    single_file_output: Option<&Path>,
    output_root: Option<&Path>,
) -> error::Result<bool> {
    let ext_config = ExtensionConfig::new(cli.extra_extensions.iter().cloned());
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let kind = ext_config
        .doc_kind(ext)
        .unwrap_or(config::DocKind::PlainText);

    let source = io::read_to_string(path)?;
    let result = processor::process(&source, kind);

    if cli.diff && result.changed {
        diff::print_diff(&path.display().to_string(), &source, &result.output);
    }

    if cli.dry_run {
        return Ok(result.changed);
    }

    if cli.in_place {
        if result.changed {
            io::write_string(path, &result.output)?;
        }
        return Ok(result.changed);
    }

    if let Some(out_file) = single_file_output {
        io::write_string(out_file, &result.output)?;
        return Ok(result.changed);
    }

    if let Some(root) = output_root {
        let dest = mirror_path(path, root);
        io::write_string(&dest, &result.output)?;
        return Ok(result.changed);
    }

    if !cli.diff {
        io::write_stdout(&result.output)?;
    }
    Ok(result.changed)
}

/// For directory-tree processing with `-o <dir>`, mirror the file under
/// the given root. This is intentionally simple (flattens by file name)
/// rather than reconstructing the full relative path, since `uztrans`
/// doesn't currently track which of possibly multiple `paths` roots a
/// given file came from; documented as a known limitation in the README.
fn mirror_path(path: &Path, root: &Path) -> PathBuf {
    match path.file_name() {
        Some(name) => root.join(name),
        None => root.join(path),
    }
}

//! LevitateOS rootfs test runner.
//!
//! Tests the rootfs as a daily driver OS using systemd-nspawn.
//! Each test answers: "Can a user do X with this OS?"

mod container;
mod tests;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::time::Instant;

use container::Container;
use tests::{TestResult, all_tests};

#[derive(Parser)]
#[command(name = "rootfs-tests")]
#[command(about = "Daily driver tests for LevitateOS rootfs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests
    Run {
        /// Path to rootfs tarball
        #[arg(long, conflicts_with = "rootfs")]
        tarball: Option<PathBuf>,

        /// Use an existing rootfs without extracting or deleting it
        #[arg(long, conflicts_with = "tarball")]
        rootfs: Option<PathBuf>,

        /// Run only tests in a specific category
        #[arg(long)]
        category: Option<String>,

        /// Show detailed output for each test
        #[arg(long, short)]
        verbose: bool,
    },

    /// List all available tests
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            tarball,
            rootfs,
            category,
            verbose,
        } => run_tests(tarball.as_deref(), rootfs.as_deref(), category, verbose),
        Commands::List => list_tests(),
    }
}

fn run_tests(
    tarball: Option<&Path>,
    rootfs: Option<&Path>,
    category: Option<String>,
    verbose: bool,
) -> Result<()> {
    let all = all_tests();
    let categories: std::collections::BTreeSet<_> =
        all.iter().map(|test| test.category().to_string()).collect();
    let tests: Vec<_> = if let Some(ref cat) = category {
        all.into_iter()
            .filter(|test| test.category() == cat)
            .collect()
    } else {
        all
    };
    if tests.is_empty() {
        bail!(
            "rootfs-tests: no runtime checks selected for category {:?}; available categories: {}. Run rootfs-tests list or choose --category <category>",
            category,
            categories.into_iter().collect::<Vec<_>>().join(", ")
        );
    }

    println!("LevitateOS Rootfs Tests");
    println!("=======================\n");
    println!("Testing: Can a user use this as a daily driver OS?\n");

    // Each archive run owns a fresh directory; existing rootfs inputs are borrowed.
    let extracted = if rootfs.is_none() {
        Some(
            tempfile::Builder::new()
                .prefix("levitate-rootfs-test-")
                .tempdir()?,
        )
    } else {
        None
    };
    let mut container = if let Some(rootfs) = rootfs {
        println!("Using rootfs {} ...", rootfs.display());
        Container::from_dir(rootfs)?
    } else {
        let tarball =
            tarball.unwrap_or_else(|| Path::new(".artifacts/out/levitate/levitateos-base.tar.xz"));
        println!("Extracting {} ...", tarball.display());
        Container::from_tarball(tarball, extracted.as_ref().unwrap().path().join("rootfs"))?
    };
    println!("Ready.\n");

    // Run tests
    let start = Instant::now();
    let mut results: Vec<TestResult> = Vec::new();
    let mut current_category = String::new();

    for test in &tests {
        // Print category header
        if test.category() != current_category {
            if !current_category.is_empty() {
                println!();
            }
            current_category = test.category().to_string();
            println!("━━━ {} ━━━", current_category.to_uppercase());
        }

        // Run test
        let result = test.run(&container);
        print_result(&result, verbose);

        // FAIL FAST: Stop on first failure
        if !result.passed {
            println!("\n════════════════════════════════════════════════════════════");
            println!("FAIL FAST: Stopping on first failure");
            println!("════════════════════════════════════════════════════════════\n");
            println!("✗ {} FAILED", result.name);
            println!("  ensures: {}", result.ensures);
            println!("  error: {}", result.output);
            bail!(
                "rootfs-tests: runtime check '{}' failed: {}",
                result.name,
                result.output
            );
        }

        results.push(result);
    }

    // A successful run must also clean up its owned extraction successfully.
    container.cleanup()?;

    // Summary
    println!("\n════════════════════════════════════════════════════════════\n");

    println!(
        "✓ All {} selected tests passed ({:.1}s)",
        results.len(),
        start.elapsed().as_secs_f64()
    );
    println!("\nAll selected runtime checks passed.");

    Ok(())
}

fn print_result(result: &TestResult, verbose: bool) {
    let status = if result.passed { "✓" } else { "✗" };
    let time = format!("{:.1}s", result.duration.as_secs_f64());

    if result.passed {
        println!("  {} {} ({})", status, result.name, time);
        if verbose {
            println!("      ensures: {}", result.ensures);
            println!("      output: {}", truncate(&result.output, 60));
        }
    } else {
        println!("  {} {} ({}) - FAILED", status, result.name, time);
        println!("      ensures: {}", result.ensures);
        println!("      error: {}", result.output);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    let s = s.lines().next().unwrap_or(s);
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn list_tests() -> Result<()> {
    println!("LevitateOS Rootfs Tests\n");
    println!("Each test verifies the OS works for daily use.\n");

    let tests = all_tests();
    let mut current_category = String::new();

    for test in &tests {
        if test.category() != current_category {
            if !current_category.is_empty() {
                println!();
            }
            current_category = test.category().to_string();
            println!("{}:", current_category.to_uppercase());
        }
        println!("  • {}", test.name());
        println!("    ensures: {}", test.ensures());
    }

    println!("\nTotal: {} tests", tests.len());
    Ok(())
}

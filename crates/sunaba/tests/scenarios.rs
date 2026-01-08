//! Scenario integration tests
//!
//! These tests validate game mechanics through scenario execution.
//!
//! ## Test Categories
//!
//! - **Default tests**: Fast smoke tests (<3s) that run with `cargo test`
//! - **Ignored tests**: Comprehensive tests (3-10s) that run with `cargo test -- --ignored`
//!
//! ## Usage
//!
//! ```bash
//! # Run only fast smoke tests
//! cargo test --test scenarios --features headless
//!
//! # Run comprehensive scenario tests
//! cargo test --test scenarios --features headless -- --ignored
//!
//! # Run all scenario tests
//! cargo test --test scenarios --features headless -- --include-ignored
//! ```

#![cfg(feature = "headless")]

use anyhow::Result;
use sunaba::scenario::{ScenarioDefinition, ScenarioExecutor};
use sunaba_core::world::World;

/// Helper function to run a scenario test from a .ron file
/// Paths are relative to workspace root
fn run_scenario_test(path: &str) -> Result<()> {
    // Tests run from workspace root when using workspace setup
    // But when running from crate, we need to go up to workspace
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| std::path::PathBuf::from(p).parent().unwrap().parent().unwrap().to_path_buf())
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let full_path = workspace_root.join(path);

    let scenario = ScenarioDefinition::from_file(&full_path)?;
    let mut executor = ScenarioExecutor::new();
    let mut world = World::new(false);
    let report = executor.execute_scenario(&scenario, &mut world)?;

    assert!(
        report.passed,
        "Scenario '{}' failed:\n{:#?}",
        scenario.name, report.verification_failures
    );

    Ok(())
}

// =============================================================================
// FAST SMOKE TESTS (run by default)
// =============================================================================

/// Basic scenario execution smoke test
/// Verifies the scenario system works without heavy simulation
#[test]
fn test_basic_scenario_execution() -> Result<()> {
    run_scenario_test("scenarios/smoke_test.ron")
}

// =============================================================================
// COMPREHENSIVE TESTS (run with --ignored)
// =============================================================================

/// Test mining mechanics: mine stone and verify material removal
#[test]
#[ignore]
fn test_mining_mechanics() -> Result<()> {
    run_scenario_test("scenarios/test_mining.ron")
}

/// Test all scenarios in the scenarios/ directory
/// This ensures all example scenarios remain valid
#[test]
#[ignore]
fn test_all_scenario_files() -> Result<()> {
    use std::fs;

    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| std::path::PathBuf::from(p).parent().unwrap().parent().unwrap().to_path_buf())
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let scenario_dir = workspace_root.join("scenarios");
    let entries = fs::read_dir(&scenario_dir)?;

    let mut tested = 0;
    let mut failures = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "ron") {
            let path_str = path.to_str().unwrap();
            println!("Testing scenario: {}", path_str);

            match run_scenario_test(path_str) {
                Ok(()) => {
                    tested += 1;
                    println!("  ✓ Passed");
                }
                Err(e) => {
                    let err_msg = format!("{}", e);
                    println!("  ✗ Failed: {}", err_msg);
                    failures.push((path_str.to_string(), e));
                }
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("\n{} scenario(s) failed:", failures.len());
        for (path, err) in &failures {
            eprintln!("  - {}: {}", path, err);
        }
        panic!("{}/{} scenarios failed", failures.len(), tested + failures.len());
    }

    println!("\n✓ All {} scenario files passed", tested);
    Ok(())
}

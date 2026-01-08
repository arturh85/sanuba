//! Execution results and reporting

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::verification::VerificationResult;

/// Report from scenario execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// Scenario name
    pub scenario_name: String,

    /// Timestamp (ISO 8601)
    pub timestamp: String,

    /// Overall pass/fail status
    pub passed: bool,

    /// Total frames executed
    pub frames_executed: usize,

    /// Number of actions executed
    pub actions_executed: usize,

    /// Verification failures (empty if all passed)
    pub verification_failures: Vec<VerificationResult>,

    /// Execution log messages
    pub log: Vec<String>,

    /// Screenshot file paths
    pub screenshots: Vec<String>,
}

impl ExecutionReport {
    /// Create new execution report
    pub fn new(scenario_name: String) -> Self {
        Self {
            scenario_name,
            timestamp: chrono::Utc::now().to_rfc3339(),
            passed: false,
            frames_executed: 0,
            actions_executed: 0,
            verification_failures: Vec::new(),
            log: Vec::new(),
            screenshots: Vec::new(),
        }
    }

    /// Check if all verifications passed
    pub fn success(&self) -> bool {
        self.verification_failures.is_empty()
    }

    /// Save report to JSON file
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize execution report to JSON")?;

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        std::fs::write(path.as_ref(), json).with_context(|| {
            format!(
                "Failed to write execution report: {}",
                path.as_ref().display()
            )
        })?;

        Ok(())
    }

    /// Load report from JSON file
    pub fn from_json(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read report file: {}", path.as_ref().display()))?;

        let report = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON report: {}", path.as_ref().display()))?;

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_report_json() {
        let mut report = ExecutionReport::new("Test Scenario".to_string());
        report.passed = true;
        report.frames_executed = 120;
        report.actions_executed = 5;
        report.log.push("Test log message".to_string());

        // Test JSON serialization
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("Test Scenario"));
        assert!(json.contains("\"frames_executed\": 120"));

        // Test round-trip
        let deserialized: ExecutionReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scenario_name, report.scenario_name);
        assert_eq!(deserialized.frames_executed, report.frames_executed);
    }

    #[test]
    fn test_success_check() {
        let mut report = ExecutionReport::new("Test".to_string());
        assert!(report.success(), "Should succeed with no failures");

        report.verification_failures.push(VerificationResult {
            passed: false,
            message: "Test failure".to_string(),
            actual_value: None,
        });
        assert!(!report.success(), "Should fail with verification failures");
    }
}

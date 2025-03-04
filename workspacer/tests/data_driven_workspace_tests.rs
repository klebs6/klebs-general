// ---------------- [ File: tests/data_driven_workspace_tests.rs ]
// ---------------- [ File: tests/data_driven_workspace_tests.rs ]
// tests/data_driven_workspace_tests.rs

use crate::mock::create_mock_workspace;
use crate::workspace::Workspace;
use crate::errors::WorkspaceError;
use serde::Deserialize;
use tokio::fs;

#[derive(Debug, Deserialize)]
struct WorkspaceScenario {
    name: String,
    crate_configs: Vec<CrateScenario>,
    expect_error: bool,
}

// Minimal info about each crate
#[derive(Debug, Deserialize)]
struct CrateScenario {
    name: String,
    add_readme: bool,
    add_src: bool,
}

#[cfg(test)]
mod data_driven {
    use super::*;

    #[tokio::test]
    async fn test_data_driven_workspace_scenarios() -> Result<(), WorkspaceError> {
        // 1) Load scenarios from a local JSON/YAML
        let scenarios_str = fs::read_to_string("tests/workspace_scenarios.json").await?;
        let scenarios: Vec<WorkspaceScenario> = serde_json::from_str(&scenarios_str)
            .expect("failed to parse JSON scenarios");

        // 2) For each scenario, create & validate
        for scenario in scenarios {
            println!("Testing scenario: {}", scenario.name);
            let crate_configs = scenario.crate_configs.iter().map(|c| {
                let mut cfg = crate::mock::CrateConfig::new(&c.name);
                if c.add_readme { cfg = cfg.with_readme(); }
                if c.add_src { cfg = cfg.with_src_files(); }
                cfg
            }).collect::<Vec<_>>();

            let ws_path = create_mock_workspace(crate_configs).await?;

            let result = Workspace::new(&ws_path).await;
            match (result, scenario.expect_error) {
                (Ok(_), true) => panic!("Scenario '{}' expected an error but got Ok", scenario.name),
                (Err(_), false) => panic!("Scenario '{}' expected Ok but got an error", scenario.name),
                _ => {} // match
            }
        }

        Ok(())
    }
}
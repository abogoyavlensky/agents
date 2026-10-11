//! `cargo xtask <task>`: the project's quality gates, one command for people, agents and CI.
//!
//! - `ci`: fmt --check, clippy -D warnings (including the no-panic lints), layers, tests
//! - `layers`: crates below the UI must not depend on a UI toolkit

use std::process::{Command, ExitCode};

/// Workspace crates that must stay UI-free, and the crates they must not depend on.
const UI_FREE: &[&str] = &["myapp-core"];
const UI_CRATES: &[&str] = &["egui", "eframe", "egui-winit", "winit", "egui-wgpu", "wgpu", "rfd"];

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = match task.as_str() {
        "ci" => ci(),
        "layers" => layers(),
        _ => Err("usage: cargo xtask <ci|layers>".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cargo(args: &[&str]) -> Result<(), String> {
    eprintln!("$ cargo {}", args.join(" "));
    let status = Command::new(env!("CARGO"))
        .args(args)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed", args.join(" ")))
    }
}

fn ci() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    layers()?;
    cargo(&["test", "--workspace"])
}

fn layers() -> Result<(), String> {
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1"])
        .output()
        .map_err(|e| e.to_string())?;
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let packages = meta["packages"].as_array().cloned().unwrap_or_default();
    let resolve = meta["resolve"]["nodes"].as_array().cloned().unwrap_or_default();
    let name_of = |id: &str| {
        packages
            .iter()
            .find(|p| p["id"] == id)
            .and_then(|p| p["name"].as_str())
            .unwrap_or("")
            .to_string()
    };
    let mut bad = Vec::new();
    for krate in UI_FREE {
        let Some(pkg) = packages.iter().find(|p| p["name"] == *krate) else {
            continue;
        };
        // Walk the full (normal) dependency closure.
        let mut stack = vec![pkg["id"].as_str().unwrap_or("").to_string()];
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = stack.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let Some(node) = resolve.iter().find(|n| n["id"] == id.as_str()) else {
                continue;
            };
            for dep in node["deps"].as_array().into_iter().flatten() {
                let normal = dep["dep_kinds"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|k| k["kind"].is_null());
                let Some(dep_id) = dep["pkg"].as_str() else { continue };
                if !normal {
                    continue;
                }
                let name = name_of(dep_id);
                if UI_CRATES.contains(&name.as_str()) {
                    bad.push(format!("{krate} depends on {name}"));
                }
                stack.push(dep_id.to_string());
            }
        }
    }
    if bad.is_empty() {
        eprintln!("layers: ok");
        Ok(())
    } else {
        Err(format!("layer violations:\n  {}", bad.join("\n  ")))
    }
}

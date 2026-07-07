use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

mod extract;
mod generate;

use extract::extract_crate;
use generate::{generate_json, generate_mdx};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    crates: Vec<CrateConfig>,
    output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct CrateConfig {
    name: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputConfig {
    json_dir: String,
    mdx_dir: String,
}

fn main() -> Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let app_path = workspace_root.join("app");
    let signaling_path = workspace_root.join("signaling");

    let config = Config {
        crates: vec![
            CrateConfig {
                name: "tabletop".to_string(),
                path: app_path.to_string_lossy().to_string(),
            },
            CrateConfig {
                name: "signaling".to_string(),
                path: signaling_path.to_string_lossy().to_string(),
            },
        ],
        output: OutputConfig {
            json_dir: workspace_root.join("docs/_data").to_string_lossy().to_string(),
            mdx_dir: workspace_root.join("docs/content/docs").to_string_lossy().to_string(),
        },
    };

    fs::create_dir_all(&config.output.json_dir)?;
    fs::create_dir_all(&config.output.mdx_dir)?;

    let mut all_crates = HashMap::new();

    for crate_cfg in &config.crates {
        println!("Extraindo: {} ({})", crate_cfg.name, crate_cfg.path);
        let crate_data = extract_crate(&crate_cfg.name, &crate_cfg.path)?;
        all_crates.insert(crate_cfg.name.clone(), crate_data);
    }

    generate_json(&all_crates, &config.output.json_dir)?;
    generate_mdx(&all_crates, &config.output.mdx_dir)?;

    println!("Documentação gerada em {}", config.output.json_dir);
    println!("MDX gerado em {}", config.output.mdx_dir);

    Ok(())
}
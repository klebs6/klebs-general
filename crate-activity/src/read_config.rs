crate::ix!();

pub const DEFAULT_USER_AGENT: &'static str = "crate-activity-bot/1.0 (contact@example.com)";

// Read crate names from a config file (~/.published-crates)
pub async fn read_crate_list(config_dir: &Path) -> Vec<String> {
    let crate_list_file = config_dir.join("crate_list.txt");

    if let Ok(file) = File::open(&crate_list_file).await {
        let mut lines = BufReader::new(file).lines();
        let mut crate_list = Vec::new();

        while let Some(line) = lines.next_line().await.unwrap_or(None) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                crate_list.push(trimmed.to_string());
            }
        }
        crate_list
    } else {
        eprintln!("Warning: Could not find {}, using default crate list.", crate_list_file.display());
        vec![
            "serde".to_string(),
            "tokio".to_string(),
        ]
    }
}

pub async fn read_user_agent(config_dir: &Path) -> String {
    let user_agent_file = config_dir.join("user_agent.txt");

    if let Ok(contents) = fs::read_to_string(&user_agent_file).await {
        contents.trim().to_string()
    } else {
        eprintln!(
            "Warning: Could not find {}, using default user agent.",
            user_agent_file.display()
        );
        DEFAULT_USER_AGENT.to_string()
    }
}

pub async fn ensure_config_structure(config_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(config_dir.join("cache")).await?;
    if !config_dir.join("crate_list.txt").exists() {
        fs::write(config_dir.join("crate_list.txt"), "serde\ntokio\n").await?;
    }
    if !config_dir.join("user_agent.txt").exists() {
        fs::write(config_dir.join("user_agent.txt"), DEFAULT_USER_AGENT).await?;
    }
    Ok(())
}

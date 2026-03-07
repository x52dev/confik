use std::{fs, path::PathBuf, sync::OnceLock};

use confik::{Configuration, FileSource, ReloadableConfig};

#[derive(Configuration, Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
}

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

impl ReloadableConfig for ServerConfig {
    type Error = anyhow::Error;

    fn build() -> Result<Self, Self::Error> {
        let config = Self::builder()
            .override_with(FileSource::new(
                CONFIG_PATH
                    .get()
                    .expect("CONFIG_PATH not initialized")
                    .as_path(),
            ))
            .try_build()?;

        if config.max_connections > 1_000 {
            anyhow::bail!("max_connections must be <= 1000");
        }

        Ok(config)
    }
}

fn main() {
    println!("=== Hot-Reloadable Configuration Example ===\n");

    // Initialize the OnceLock statics
    let config_file = tempfile::Builder::new()
        .suffix(".toml")
        .tempfile()
        .expect("Failed to create temp file");
    let config_path = CONFIG_PATH.get_or_init(|| config_file.path().to_path_buf());

    // Initialize with the first configuration
    fs::write(
        config_path,
        r#"
        host = "localhost"
        port = 8080
        max_connections = 100
        "#,
    )
    .expect("Failed to write initial config");
    println!("Created config file at: {:?}\n", config_path);

    // Create a reloading config using the convenient method
    let config = ServerConfig::reloading().expect("Failed to load initial config");

    // Load and use the current configuration
    let current = config.load();
    println!("Initial configuration:");
    println!("  Host: {}", current.host);
    println!("  Port: {}", current.port);
    println!("  Max Connections: {}", current.max_connections);

    // Add a callback to be notified when config reloads
    let config_with_callback = config.with_on_update(|| {
        println!("✓ Configuration reloaded successfully!");
    });

    // Modify the config file
    println!("\n--- Modifying the config file ---");
    fs::write(
        config_path,
        r#"
        host = "0.0.0.0"
        port = 80
        max_connections = 10
        "#,
    )
    .expect("Failed to write updated config");
    println!("Updated config file with new values");

    println!("\n--- Reloading configuration ---");
    config_with_callback
        .reload()
        .expect("Failed to reload config");

    // The configuration is atomically updated with new values!
    let updated = config_with_callback.load();
    println!("After reload:");
    println!("  Host: {}", updated.host);
    println!("  Port: {}", updated.port);
    println!("  Max Connections: {}", updated.max_connections);

    // Verify the values actually changed
    assert_eq!(updated.host, "0.0.0.0");
    assert_eq!(updated.port, 80);
    assert_eq!(updated.max_connections, 10);
    println!("\n✓ Configuration values changed successfully!");

    // Write invalid config
    fs::write(
        config_path,
        r#"
        host = "localhost"
        port = 8080
        max_connections = 99999999
        "#,
    )
    .expect("Failed to write updated config");
    println!("Updated config file with new values");

    println!("\n--- Reloading configuration ---");
    config_with_callback
        .reload()
        .expect_err("Invalid config rejected");

    // Verify the values actually changed
    assert_eq!(updated.host, "0.0.0.0");
    assert_eq!(updated.port, 80);
    assert_eq!(updated.max_connections, 10);

    println!("\n=== Example complete ===");
}

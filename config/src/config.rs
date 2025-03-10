//! Configuration module for parsing YAML configuration files.

use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_yaml;
use std::error::Error;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

/// Main configuration struct that holds all configuration options for the application.
#[derive(Deserialize, Debug, Default)]
pub struct CliConfig {
    /// IO configuration defining input/output-related parameters.
    #[serde(default)]
    pub io: CliConfigIO,
    /// Worker configuration, setting parameters for worker threads.
    #[serde(default)]
    pub workers: CliConfigWorkers,
    /// Ruleset configuration, including paths to GeoIP and GeoSite files.
    #[serde(default)]
    pub ruleset: CliConfigRuleset,
    /// Replay configuration, specifying whether real-time replay is enabled.
    #[serde(default)]
    pub replay: CliConfigReplay,
}

/// IO configuration struct, containing settings for queues and buffers.
#[derive(Deserialize, Debug)]
pub struct CliConfigIO {
    /// Size of the queue used in IO operations.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub queue_size: u32,
    /// Size of the receive buffer in bytes.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub rcv_buf: i32,
    /// Size of the send buffer in bytes.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub snd_buf: i32,
    /// Enables or disables local mode.
    /// Set to false if you want to run gfw-rs on FORWARD chain (e.g. on a router)
    #[serde(default)]
    pub local: bool,
    /// Enables or disables the RST (Reset) functionality.
    /// Set to true if you want to send RST for blocked TCP connections, local=false only
    #[serde(default)]
    pub rst: bool,
}

/// Replay configuration struct, defining the mode for replay.
#[derive(Deserialize, Debug, Default)]
pub struct CliConfigReplay {
    /// Specifies if real-time replay mode is enabled.
    /// Set to true if you want to replay the packets in the pcap file in "real time" (instead of as fast as possible)
    #[serde(default)]
    pub realtime: bool,
}

/// Worker configuration struct, containing thread count, buffer, and timeout settings.
#[derive(Deserialize, Debug)]
pub struct CliConfigWorkers {
    /// Number of worker threads.
    /// Recommended to be no more than the number of CPU cores
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub count: usize,
    /// Size of the queue for each worker.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub queue_size: u32,
    /// Maximum number of buffered pages across all TCP connections.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub tcp_max_buffered_pages_total: u32,
    /// Maximum number of buffered pages per TCP connection.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub tcp_max_buffered_pages_per_conn: u32,
    /// TCP timeout duration, the unit is 's'.
    /// How long a connection is considered dead when no data is being transferred.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub tcp_timeout: u64,
    /// Maximum number of UDP streams.
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub udp_max_streams: u32,
}

/// Ruleset configuration struct, containing paths to GeoIP and GeoSite files.
#[derive(Deserialize, Debug)]
pub struct CliConfigRuleset {
    /// Path to the GeoIP file.
    #[serde(default)]
    pub geoip: String,
    /// Path to the GeoSite file.
    #[serde(default)]
    pub geosite: String,
}

/// Parses YAML configuration from a string and returns a `CliConfig` struct.
#[allow(unused)]
fn load_config_from_string(yaml_str: &str) -> Result<CliConfig, Box<dyn Error>> {
    // Parse the YAML string into the CliConfig struct
    let config: CliConfig = serde_yaml::from_str(yaml_str)?;
    Ok(config)
}

/// Parses YAML configuration from a file and returns a `CliConfig` struct.
pub async fn load_config_from_file(file_path: &str) -> Result<CliConfig, Box<dyn Error>> {
    // Open the file asynchronously
    let file = File::open(file_path).await?;

    // Wrap the file in a BufReader for efficient reading
    let reader = BufReader::new(file);

    // Read the entire file into a string
    let mut contents = String::new();
    let mut reader = reader;
    reader.read_to_string(&mut contents).await?;

    // Parse the YAML string into the CliConfig struct
    let config: CliConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

/// Provide default values by implementing `Default` trait for each struct
impl Default for CliConfigIO {
    fn default() -> Self {
        CliConfigIO {
            queue_size: 1024,
            rcv_buf: 4194304,
            snd_buf: 4194304,
            local: true,
            rst: false,
        }
    }
}

impl Default for CliConfigWorkers {
    fn default() -> Self {
        CliConfigWorkers {
            count: 4,
            queue_size: 64,
            tcp_max_buffered_pages_total: 65536,
            tcp_max_buffered_pages_per_conn: 16,
            tcp_timeout: 600, // seconds
            udp_max_streams: 4096,
        }
    }
}

impl Default for CliConfigRuleset {
    fn default() -> Self {
        CliConfigRuleset {
            geoip: "./geoip".to_string(),
            geosite: "./geosite".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_from_string() {
        let yaml_data = r#"
        io:
          queue_size: 1024
          rcv_buf: 4194304
          snd_buf: 4194304
          local: true
          rst: false

        workers:
          count: 4
          queue_size: 64
          tcp_max_buffered_pages_total: 65536
          tcp_max_buffered_pages_per_conn: 16
          tcp_timeout: 600
          udp_max_streams: 4096

        ruleset:
          geoip: "/path/to/geoip"
          geosite: "/path/to/geosite"

        replay:
          realtime: false
        "#;

        // Attempt to load the configuration from the YAML string
        let config = load_config_from_string(yaml_data).expect("Failed to parse YAML");

        // Check IO configuration
        assert_eq!(config.io.queue_size, 1024);
        assert_eq!(config.io.rcv_buf, 4194304);
        assert_eq!(config.io.snd_buf, 4194304);
        assert_eq!(config.io.local, true);
        assert_eq!(config.io.rst, false);

        // Check Workers configuration
        assert_eq!(config.workers.count, 4);
        assert_eq!(config.workers.queue_size, 64);
        assert_eq!(config.workers.tcp_max_buffered_pages_total, 65536);
        assert_eq!(config.workers.tcp_max_buffered_pages_per_conn, 16);
        assert_eq!(config.workers.udp_max_streams, 4096);
        assert_eq!(config.workers.tcp_timeout, 600); // 10 minutes in seconds

        // Check Ruleset configuration
        assert_eq!(config.ruleset.geoip, "/path/to/geoip");
        assert_eq!(config.ruleset.geosite, "/path/to/geosite");

        // Check Replay configuration
        assert_eq!(config.replay.realtime, false);
    }
}

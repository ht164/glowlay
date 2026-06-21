use serde::Deserialize;
use std::fs;
use std::path::Path;

fn default_anchor() -> String {
    "top-left".to_string()
}

fn default_bg_alpha() -> f32 {
    0.4
}

fn default_font_scale() -> f32 {
    1.2
}

#[derive(Debug, Deserialize, Clone)]
pub struct StyleConfig {
    #[serde(default = "default_bg_alpha")]
    pub bg_alpha: f32,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_anchor")]
    pub anchor: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitorConfig {
    pub update_interval_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct FeaturesConfig {
    pub enable_cpu: bool,
    pub enable_mem: bool,
    pub enable_gpu: bool,
    pub enable_storage: bool,
    pub enable_network: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub interface: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    pub paths: Vec<String>, // 複数のパスを指定可能にする
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub window: WindowConfig,
    pub monitor: MonitorConfig,
    pub features: FeaturesConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub style: StyleConfig,
}

impl Config {
    pub fn load() -> Self {
        let mut paths = vec![
            "./config.toml".to_string(),
            "~/.config/glowlay/config.toml".to_string(),
            "/etc/glowlay/config.toml".to_string(),
        ];

        // 1. 実行可能バイナリが存在するディレクトリの config.toml を最優先で探索
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let exe_config = exe_dir.join("config.toml");
                if let Some(path_str) = exe_config.to_str() {
                    paths.insert(0, path_str.to_string());
                }
                
                // cargo run 時のワークスペースルートに配慮
                if let Some(project_root) = exe_dir.parent().and_then(|p| p.parent()) {
                    let project_config = project_root.join("config.toml");
                    if let Some(path_str) = project_config.to_str() {
                        paths.push(path_str.to_string());
                    }
                }
            }
        }


        for path_str in paths.iter() {
            let p = if path_str.starts_with("~") {
                if let Some(home) = std::env::var_os("HOME") {
                    Path::new(&home).join(&path_str[2..])
                } else {
                    Path::new(path_str).to_path_buf()
                }
            } else {
                Path::new(path_str).to_path_buf()
            };

            if p.exists() {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(config) = toml::from_str(&content) {
                        println!("Loaded config from: {:?}", p);
                        return config;
                    }
                }
            }
        }

        // フォールバック用のデフォルト設定
        Config {
            window: WindowConfig {
                width: 240,
                height: 350,
                x: 40,
                y: 40,
                anchor: "top-left".to_string(),
            },
            monitor: MonitorConfig {
                update_interval_ms: 1000,
            },
            features: FeaturesConfig {
                enable_cpu: true,
                enable_mem: true,
                enable_gpu: true,
                enable_storage: true,
                enable_network: true,
            },
            network: NetworkConfig {
                interface: String::new(),
            },
            storage: StorageConfig {
                paths: vec!["/".to_string()],
            },
            style: StyleConfig {
                bg_alpha: 0.4,
                font_scale: 1.2,
            },
        }
    }
}

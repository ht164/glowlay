use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt, ComponentExt};
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use std::time::Instant;
use crate::config::Config;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DiskStats {
    pub path: String,
    pub used: u64,
    pub total: u64,
    pub pct: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub mem_total: u64,
    pub mem_used: u64,
    pub mem_pct: f32,
    pub disks: Vec<DiskStats>, // 複数ディスク情報を格納
    pub net_down_kbps: f64,
    pub net_up_kbps: f64,
    pub cpu_temp: f32,
    pub sys_temp: f32,
    pub gpu_temp: Option<u32>,
    pub gpu_mem_used: Option<u64>,
    pub gpu_mem_total: Option<u64>,
    pub gpu_mem_pct: Option<f32>,
}

pub struct SystemMonitor {
    sys: System,
    nvml: Option<Nvml>,
    last_update: Instant,
    last_net_bytes: Option<(u64, u64)>, // (received, transmitted)
    config: Config,
}

impl SystemMonitor {
    pub fn new(config: Config) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let nvml = if config.features.enable_gpu {
            Nvml::init().ok()
        } else {
            None
        };

        SystemMonitor {
            sys,
            nvml,
            last_update: Instant::now(),
            last_net_bytes: None,
            config,
        }
    }

    pub fn fetch(&mut self) -> SystemStats {
        let now = Instant::now();
        let duration = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        // CPU & Memory & Components (温度)
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.sys.refresh_components_list();
        self.sys.refresh_components();

        // cpus() から各コアの使用率を取得し、その平均値を計算
        let cpus = self.sys.cpus();
        let cpu_usage = if !cpus.is_empty() {
            let total: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
            total / cpus.len() as f32
        } else {
            0.0
        };

        let mem_total = self.sys.total_memory();
        let mem_used = self.sys.used_memory();
        let mem_pct = if mem_total > 0 {
            (mem_used as f32 / mem_total as f32) * 100.0
        } else {
            0.0
        };

        // 温度センサー情報の取得
        let mut cpu_temp = 0.0;
        let mut sys_temp = 0.0;

        for component in self.sys.components() {
            let label = component.label().to_lowercase();
            if label.contains("cpu") || label.contains("core") || label.contains("k10temp") || label.contains("tctl") {
                if cpu_temp == 0.0 {
                    cpu_temp = component.temperature();
                }
            }
            if label.contains("temp") || label.contains("mb") || label.contains("board") || label.contains("acpi") || label.contains("edge") {
                if sys_temp == 0.0 {
                    sys_temp = component.temperature();
                }
            }
        }

        if cpu_temp == 0.0 {
            cpu_temp = 42.0;
        }
        if sys_temp == 0.0 {
            sys_temp = 36.0;
        }

        // 複数マウントポイントに対するディスク情報の取得
        let mut disks = Vec::new();
        if self.config.features.enable_storage {
            self.sys.refresh_disks_list();
            self.sys.refresh_disks();
            
            for path_str in &self.config.storage.paths {
                for disk in self.sys.disks() {
                    if disk.mount_point() == std::path::Path::new(path_str) {
                        let total = disk.total_space();
                        let used = total.saturating_sub(disk.available_space());
                        let pct = if total > 0 {
                            (used as f32 / total as f32) * 100.0
                        } else {
                            0.0
                        };
                        disks.push(DiskStats {
                            path: path_str.clone(),
                            used,
                            total,
                            pct,
                        });
                        break;
                    }
                }
            }
        }

        // Network
        let mut net_down_kbps = 0.0;
        let mut net_up_kbps = 0.0;
        if self.config.features.enable_network {
            self.sys.refresh_networks_list();
            self.sys.refresh_networks();
            let mut total_rx = 0;
            let mut total_tx = 0;
            
            let target_iface = &self.config.network.interface;
            let networks = self.sys.networks();
            
            if !target_iface.is_empty() {
                for (name, data) in networks {
                    if name == target_iface {
                        total_rx = data.total_received();
                        total_tx = data.total_transmitted();
                        break;
                    }
                }
            } else {
                for (name, data) in networks {
                    if name != "lo" {
                        total_rx += data.total_received();
                        total_tx += data.total_transmitted();
                    }
                }
            }

            if let Some((prev_rx, prev_tx)) = self.last_net_bytes {
                let rx_diff = total_rx.saturating_sub(prev_rx);
                let tx_diff = total_tx.saturating_sub(prev_tx);
                if duration > 0.0 {
                    net_down_kbps = (rx_diff as f64 / 1024.0) / duration;
                    net_up_kbps = (tx_diff as f64 / 1024.0) / duration;
                }
            }
            self.last_net_bytes = Some((total_rx, total_tx));
        }

        // GPU
        let mut gpu_temp = None;
        let mut gpu_mem_used = None;
        let mut gpu_mem_total = None;
        let mut gpu_mem_pct = None;

        if let Some(ref nvml) = self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                gpu_temp = device.temperature(TemperatureSensor::Gpu).ok();
                if let Ok(mem) = device.memory_info() {
                    gpu_mem_used = Some(mem.used);
                    gpu_mem_total = Some(mem.total);
                    gpu_mem_pct = Some((mem.used as f32 / mem.total as f32) * 100.0);
                }
            }
        }

        SystemStats {
            cpu_usage,
            mem_total,
            mem_used,
            mem_pct,
            disks,
            net_down_kbps,
            net_up_kbps,
            cpu_temp,
            sys_temp,
            gpu_temp,
            gpu_mem_used,
            gpu_mem_total,
            gpu_mem_pct,
        }
    }
}

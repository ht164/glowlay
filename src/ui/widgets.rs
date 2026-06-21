use gtk4::prelude::*;
use gtk4::{DrawingArea, Box, Orientation, Label};
use gtk4::cairo;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;
use crate::monitor::SystemStats;
use crate::config::Config;

const MAX_HISTORY: usize = 60;

pub struct Dashboard {
    pub container: Box,
    cpu_graph: DrawingArea,
    cpu_val_label: Label,
    storage_labels: Vec<Label>,
    net_down_val: Label, // ダウンロード表示用
    net_up_val: Label,   // アップロード表示用
    state: Rc<RefCell<DashboardState>>,
}

struct DashboardState {
    stats: SystemStats,
    cpu_history: VecDeque<f32>,
    initialized: bool,
}

// 桁数を固定して等幅表示での文字揺れを防ぐためのフォーマット
fn format_speed(kbps: f64) -> String {
    if kbps >= 1024.0 {
        // 例: "  1.2 M", " 15.4 M" などのように常に5文字+スペース+1文字で固定長にする
        format!("{:>5.1} M", kbps / 1024.0)
    } else {
        // 例: "   10 K", "  250 K"
        format!("{:>5.0} K", kbps)
    }
}

impl Dashboard {
    pub fn new(config: &Config) -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .margin_top(0)
            .margin_bottom(0)
            .margin_start(0)
            .margin_end(0)
            .build();
        container.style_context().add_class("dashboard-container");

        let state = Rc::new(RefCell::new(DashboardState {
            stats: SystemStats::default(),
            cpu_history: {
                let mut d = VecDeque::with_capacity(MAX_HISTORY);
                for _ in 0..MAX_HISTORY { d.push_back(0.0); }
                d
            },
            initialized: false,
        }));

        let font_scale = config.style.font_scale as f64;

        // タイトル
        let title_box = Box::new(Orientation::Vertical, 0);
        let title_label = Label::builder()
            .label("SYSTEM MONITOR")
            .halign(gtk4::Align::Start)
            .build();
        title_label.style_context().add_class("dashboard-title");
        let subtitle_label = Label::builder()
            .label("DESKTOP STATUS")
            .halign(gtk4::Align::Start)
            .build();
        subtitle_label.style_context().add_class("dashboard-subtitle");
        title_box.append(&title_label);
        title_box.append(&subtitle_label);
        container.append(&title_box);

        // RAM & VRAM メーター
        let meters_box = Box::new(Orientation::Horizontal, 10);
        meters_box.set_homogeneous(true);

        // RAM メーター
        let ram_box = Box::new(Orientation::Vertical, 2);
        let ram_label = Label::new(Some("RAM USAGE"));
        ram_label.style_context().add_class("widget-label");
        let ram_meter = DrawingArea::new();
        ram_meter.set_content_width(90);
        ram_meter.set_content_height(90);
        
        let state_clone = Rc::clone(&state);
        ram_meter.set_draw_func(move |_area, cr, width, height| {
            let state = state_clone.borrow();
            let pct = state.stats.mem_pct / 100.0;
            let used_gb = state.stats.mem_used as f64 / 1024.0 / 1024.0 / 1024.0;
            let total_gb = state.stats.mem_total as f64 / 1024.0 / 1024.0 / 1024.0;

            let xc = width as f64 / 2.0;
            let yc = height as f64 / 2.0;
            let radius = f64::min(xc, yc) - 6.0;

            cr.set_source_rgba(0.1, 0.15, 0.2, 0.5);
            cr.set_line_width(5.0);
            cr.arc(xc, yc, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().unwrap();

            if pct > 0.0 {
                cr.set_source_rgba(0.0, 0.8, 1.0, 0.9);
                cr.set_line_width(5.0);
                let start_angle = -std::f64::consts::FRAC_PI_2;
                let end_angle = start_angle + (2.0 * std::f64::consts::PI * pct as f64);
                cr.arc(xc, yc, radius, start_angle, end_angle);
                cr.stroke().unwrap();
            }

            cr.set_source_rgb(0.9, 0.9, 0.9);
            cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(14.0 * font_scale);
            let text = format!("{:.0}%", state.stats.mem_pct);
            let extents = cr.text_extents(&text).unwrap();
            cr.move_to(xc - extents.width() / 2.0 - extents.x_bearing(), yc - extents.y_bearing() / 2.0 - 6.0 * font_scale);
            cr.show_text(&text).unwrap();

            cr.set_source_rgb(0.6, 0.7, 0.8);
            cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            cr.set_font_size(8.0 * font_scale);
            let detail = format!("{:.1}G/{:.0}G", used_gb, total_gb);
            let extents_d = cr.text_extents(&detail).unwrap();
            cr.move_to(xc - extents_d.width() / 2.0 - extents_d.x_bearing(), yc - extents_d.y_bearing() / 2.0 + 10.0 * font_scale);
            cr.show_text(&detail).unwrap();
        });

        ram_box.append(&ram_label);
        ram_box.append(&ram_meter);
        meters_box.append(&ram_box);

        // VRAM メーター
        let gpu_box = Box::new(Orientation::Vertical, 2);
        let gpu_label = Label::new(Some("VRAM USAGE"));
        gpu_label.style_context().add_class("widget-label");
        let gpu_meter = DrawingArea::new();
        gpu_meter.set_content_width(90);
        gpu_meter.set_content_height(90);

        let state_clone = Rc::clone(&state);
        let enable_gpu = config.features.enable_gpu;
        gpu_meter.set_draw_func(move |_area, cr, width, height| {
            if !enable_gpu {
                return;
            }
            let state = state_clone.borrow();
            let pct = state.stats.gpu_mem_pct.unwrap_or(0.0) / 100.0;
            let vram_pct = state.stats.gpu_mem_pct.unwrap_or(0.0);

            let xc = width as f64 / 2.0;
            let yc = height as f64 / 2.0;
            let radius = f64::min(xc, yc) - 6.0;

            cr.set_source_rgba(0.1, 0.15, 0.2, 0.5);
            cr.set_line_width(5.0);
            cr.arc(xc, yc, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().unwrap();

            if pct > 0.0 {
                cr.set_source_rgba(0.5, 0.0, 1.0, 0.9);
                cr.set_line_width(5.0);
                let start_angle = -std::f64::consts::FRAC_PI_2;
                let end_angle = start_angle + (2.0 * std::f64::consts::PI * pct as f64);
                cr.arc(xc, yc, radius, start_angle, end_angle);
                cr.stroke().unwrap();
            }

            cr.set_source_rgb(0.9, 0.9, 0.9);
            cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(14.0 * font_scale);
            let text = format!("{:.0}%", vram_pct);
            let extents = cr.text_extents(&text).unwrap();
            cr.move_to(xc - extents.width() / 2.0 - extents.x_bearing(), yc - extents.y_bearing() / 2.0 - 6.0 * font_scale);
            cr.show_text(&text).unwrap();

            cr.set_source_rgb(0.7, 0.6, 0.9);
            cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            cr.set_font_size(8.0 * font_scale);
            let used_mb = state.stats.gpu_mem_used.unwrap_or(0) as f64 / 1024.0 / 1024.0;
            let total_mb = state.stats.gpu_mem_total.unwrap_or(0) as f64 / 1024.0 / 1024.0;
            let detail = format!("{:.0}M/{:.0}M", used_mb, total_mb);
            let extents_d = cr.text_extents(&detail).unwrap();
            cr.move_to(xc - extents_d.width() / 2.0 - extents_d.x_bearing(), yc - extents_d.y_bearing() / 2.0 + 10.0 * font_scale);
            cr.show_text(&detail).unwrap();
        });

        gpu_box.append(&gpu_label);
        gpu_box.append(&gpu_meter);
        if config.features.enable_gpu {
            meters_box.append(&gpu_box);
        }
        container.append(&meters_box);



        // CPU 折れ線グラフ
        let cpu_box = Box::new(Orientation::Vertical, 2);
        let cpu_title_box = Box::new(Orientation::Horizontal, 0);
        let cpu_label = Label::new(Some("CPU USAGE"));
        cpu_label.style_context().add_class("widget-label");
        let cpu_val_label = Label::new(Some("0%"));
        cpu_val_label.style_context().add_class("widget-value-label");
        cpu_title_box.append(&cpu_label);
        cpu_title_box.append(&cpu_val_label);
        cpu_val_label.set_hexpand(true);
        cpu_val_label.set_halign(gtk4::Align::End);

        let cpu_graph = DrawingArea::new();
        cpu_graph.set_content_width(200);
        cpu_graph.set_content_height(60);
        cpu_graph.set_hexpand(true);

        let state_clone = Rc::clone(&state);
        cpu_graph.set_draw_func(move |_area, cr, width, height| {
            let state = state_clone.borrow();
            let w = width as f64;
            let h = height as f64;

            // 背景グリッド
            cr.set_source_rgba(0.1, 0.15, 0.2, 0.2);
            cr.set_line_width(1.0);
            for i in 1..3 {
                let y = h * (i as f64 / 3.0);
                cr.move_to(0.0, y);
                cr.line_to(w, y);
                cr.stroke().unwrap();
            }

            if state.cpu_history.is_empty() {
                return;
            }

            let step = w / (MAX_HISTORY - 1) as f64;
            let mut points = Vec::new();
            for (i, &usage) in state.cpu_history.iter().enumerate() {
                let x = i as f64 * step;
                let y = h - (usage as f64 / 100.0) * h;
                points.push((x, y));
            }

            // 1. 塗りつぶしパス
            cr.move_to(0.0, h);
            cr.line_to(points[0].0, points[0].1);
            for i in 0..(points.len() - 1) {
                let p1 = points[i];
                let p2 = points[i + 1];
                let xc = (p1.0 + p2.0) / 2.0;
                cr.curve_to(xc, p1.1, xc, p2.1, p2.0, p2.1);
            }
            cr.line_to(w, h);
            cr.close_path();

            let pat = cairo::LinearGradient::new(0.0, 0.0, 0.0, h);
            pat.add_color_stop_rgba(0.0, 0.0, 0.8, 1.0, 0.45);
            pat.add_color_stop_rgba(1.0, 0.0, 0.8, 1.0, 0.03);
            cr.set_source(&pat).unwrap();
            cr.fill().unwrap();

            // 2. 輪郭線
            cr.new_path();
            cr.move_to(points[0].0, points[0].1);
            for i in 0..(points.len() - 1) {
                let p1 = points[i];
                let p2 = points[i + 1];
                let xc = (p1.0 + p2.0) / 2.0;
                cr.curve_to(xc, p1.1, xc, p2.1, p2.0, p2.1);
            }
            cr.set_source_rgba(0.0, 0.9, 1.0, 1.0);
            cr.set_line_width(2.0);
            cr.stroke().unwrap();
        });

        cpu_box.append(&cpu_title_box);
        cpu_box.append(&cpu_graph);
        container.append(&cpu_box);



        // 温度 (TEMP) - 横方向メーターに変更
        let temp_container = Box::new(Orientation::Vertical, 2);
        temp_container.style_context().add_class("info-box-container");

        let temp_title = Label::new(Some("TEMP"));
        temp_title.style_context().add_class("widget-label");
        temp_title.set_halign(gtk4::Align::Start);
        temp_container.append(&temp_title);

        let temp_meter = DrawingArea::new();
        temp_meter.set_hexpand(true);
        temp_meter.set_content_height(60);

        let state_clone = Rc::clone(&state);
        temp_meter.set_draw_func(move |_area, cr, width, _height| {
            let state = state_clone.borrow();
            let w = width as f64;

            let bar_h = 5.0; // 横棒の太さ
            let spacing = 18.0; // 各温度バーの間隔

            let temps = [
                ("CPU", state.stats.cpu_temp as f64),
                ("SYS", state.stats.sys_temp as f64),
                ("GPU", state.stats.gpu_temp.unwrap_or(0) as f64),
            ];

            for (idx, &(label, temp)) in temps.iter().enumerate() {
                let y = 10.0 + idx as f64 * spacing;
                let max_temp = 100.0;
                let pct = f64::min(1.0, f64::max(0.0, temp / max_temp));
                
                // ラベル描画（左側）
                cr.set_source_rgb(0.6, 0.7, 0.8);
                cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                cr.set_font_size(10.0 * font_scale);
                let extents_l = cr.text_extents(label).unwrap();
                cr.move_to(5.0, y - extents_l.y_bearing() / 2.0 - bar_h / 2.0);
                cr.show_text(label).unwrap();

                // 横バーの位置 (フォントスケールに応じて左右の余白を調整)
                let start_x = 35.0 * font_scale;
                let end_x = w - 45.0 * font_scale;
                let bar_w_total = f64::max(10.0, end_x - start_x);
                let val_w = bar_w_total * pct;

                // 背景バー (薄い背景)
                cr.set_source_rgba(0.1, 0.15, 0.2, 0.5);
                cr.set_line_width(bar_h);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.move_to(start_x, y);
                cr.line_to(start_x + bar_w_total, y);
                cr.stroke().unwrap();

                // 進捗バー
                if val_w > 0.0 {
                    let pat = cairo::LinearGradient::new(start_x, 0.0, start_x + val_w, 0.0);
                    if idx == 0 {
                        // CPU: 青〜水色
                        pat.add_color_stop_rgba(0.0, 0.0, 0.8, 1.0, 0.9);
                        pat.add_color_stop_rgba(1.0, 0.0, 0.9, 1.0, 0.9);
                    } else if idx == 1 {
                        // SYS: 薄い青
                        pat.add_color_stop_rgba(0.0, 0.0, 0.7, 0.7, 0.9);
                        pat.add_color_stop_rgba(1.0, 0.0, 0.9, 0.5, 0.9);
                    } else {
                        // GPU: 緑〜オレンジ
                        pat.add_color_stop_rgba(0.0, 0.5, 0.0, 1.0, 0.9);
                        pat.add_color_stop_rgba(1.0, 0.8, 0.0, 1.0, 0.9);
                    }
                    cr.set_source(&pat).unwrap();
                    cr.set_line_width(bar_h);
                    cr.set_line_cap(cairo::LineCap::Round);
                    cr.move_to(start_x, y);
                    cr.line_to(start_x + val_w, y);
                    cr.stroke().unwrap();
                }

                // 温度値の描画（右側）
                cr.set_source_rgb(0.9, 0.9, 0.9);
                cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
                cr.set_font_size(10.0 * font_scale);
                let val_text = format!("{:.0}°C", temp);
                let extents_v = cr.text_extents(&val_text).unwrap();
                cr.move_to(w - 5.0 - extents_v.width() - extents_v.x_bearing(), y - extents_v.y_bearing() / 2.0 - bar_h / 2.0);
                cr.show_text(&val_text).unwrap();
            }
        });
        temp_container.append(&temp_meter);
        container.append(&temp_container);



        // テキスト情報 (GridレイアウトによるSTORAGE & NETWORK)
        let grid_container = Box::new(Orientation::Vertical, 2);
        grid_container.style_context().add_class("info-box-container");

        let grid = gtk4::Grid::builder()
            .row_spacing(4)
            .column_spacing(10)
            .hexpand(true)
            .build();

        let val_width = (85.0 * font_scale) as i32;
        let mut row_idx = 0;

        // 複数ディスクのマウントポイント用行
        let mut storage_labels = Vec::new();
        if config.features.enable_storage {
            for path_str in &config.storage.paths {
                let label_text = format!("STORAGE ({})", path_str);
                let storage_title = Label::new(Some(&label_text));
                storage_title.style_context().add_class("widget-label");
                storage_title.set_halign(gtk4::Align::Start);
                
                let storage_val = Label::new(Some("0G (0%)"));
                storage_val.style_context().add_class("info-value");
                storage_val.set_halign(gtk4::Align::End);
                storage_val.set_xalign(1.0);
                storage_val.set_width_request(val_width);
                
                grid.attach(&storage_title, 0, row_idx, 1, 1);
                grid.attach(&storage_val, 1, row_idx, 1, 1);
                
                storage_labels.push(storage_val);
                row_idx += 1;
            }
        }

        // ネットワーク (DOWN)
        let net_down_title = Label::new(Some("NET DOWN"));
        net_down_title.style_context().add_class("widget-label");
        net_down_title.set_halign(gtk4::Align::Start);
        let net_down_val = Label::new(Some("▼  0.0 K"));
        net_down_val.style_context().add_class("info-value");
        net_down_val.set_halign(gtk4::Align::End);
        net_down_val.set_xalign(1.0);
        net_down_val.set_width_request(val_width);
        
        grid.attach(&net_down_title, 0, row_idx, 1, 1);
        grid.attach(&net_down_val, 1, row_idx, 1, 1);
        row_idx += 1;

        // ネットワーク (UP)
        let net_up_title = Label::new(Some("NET UP"));
        net_up_title.style_context().add_class("widget-label");
        net_up_title.set_halign(gtk4::Align::Start);
        let net_up_val = Label::new(Some("▲  0.0 K"));
        net_up_val.style_context().add_class("info-value");
        net_up_val.set_halign(gtk4::Align::End);
        net_up_val.set_xalign(1.0);
        net_up_val.set_width_request(val_width);
        
        grid.attach(&net_up_title, 0, row_idx, 1, 1);
        grid.attach(&net_up_val, 1, row_idx, 1, 1);

        grid_container.append(&grid);
        container.append(&grid_container);

        Dashboard {
            container,
            cpu_graph,
            cpu_val_label,
            storage_labels,
            net_down_val,
            net_up_val,
            state,
        }
    }

    pub fn update(&self, stats: SystemStats) {
        let mut state = self.state.borrow_mut();
        state.stats = stats.clone();
        
        if !state.initialized {
            state.cpu_history.clear();
            for _ in 0..MAX_HISTORY {
                state.cpu_history.push_back(stats.cpu_usage);
            }
            state.initialized = true;
        } else {
            state.cpu_history.pop_front();
            state.cpu_history.push_back(stats.cpu_usage);
        }

        self.cpu_graph.queue_draw();
        self.container.queue_draw();

        self.cpu_val_label.set_text(&format!("{:.0}%", stats.cpu_usage));

        // 複数ストレージの値更新
        for (idx, disk) in stats.disks.iter().enumerate() {
            if idx < self.storage_labels.len() {
                let used_gb = disk.used as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_str = if used_gb >= 1024.0 {
                    format!("{:.1}T", used_gb / 1024.0)
                } else {
                    format!("{:.0}G", used_gb)
                };
                self.storage_labels[idx].set_text(&format!("{} ({:.0}%)", used_str, disk.pct));
            }
        }

        // ネットワーク (DOWN / UP それぞれ固定幅で適用)
        let down_str = format_speed(stats.net_down_kbps);
        let up_str = format_speed(stats.net_up_kbps);
        self.net_down_val.set_text(&format!("▼ {}", down_str));
        self.net_up_val.set_text(&format!("▲ {}", up_str));
    }
}

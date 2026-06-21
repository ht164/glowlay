mod config;
mod monitor;
mod ui;

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use std::cell::RefCell;
use std::rc::Rc;
use config::Config;
use monitor::SystemMonitor;
use ui::window::create_window;
use ui::widgets::Dashboard;

fn main() {
    let app = gtk4::Application::builder()
        .application_id("com.github.ht164.conky-rs")
        .build();

    app.connect_activate(|app| {
        // 設定をロード
        let config = Config::load();

        // configの設定値(bg_alpha, font_scale)に基づいて、CSSスタイルシートを動的に構築
        let bg_alpha = config.style.bg_alpha;
        let font_scale = config.style.font_scale;
        
        let font_size_subtitle = (9.0 * font_scale) as i32;
        let font_size_label = (10.0 * font_scale) as i32;
        let font_size_val_label = (12.0 * font_scale) as i32;
        
        // STORAGE, NETWORK などの数値・文字のサイズを1段階大きく調整 (10.0 -> 12.0)
        let font_size_info_val = (12.0 * font_scale) as i32;

        let css_template = include_str!("style.css");
        let css_data = css_template
            .replace("{bg_alpha}", &bg_alpha.to_string())
            .replace("{font_size_subtitle}", &font_size_subtitle.to_string())
            .replace("{font_size_label}", &font_size_label.to_string())
            .replace("{font_size_val_label}", &font_size_val_label.to_string())
            .replace("{font_size_info_val}", &font_size_info_val.to_string());

        let provider = gtk4::CssProvider::new();
        provider.load_from_data(&css_data);
        if let Some(display) = gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        // ウィンドウの作成
        let window = create_window(app, &config);

        // ダッシュボードUIの構築
        let dashboard = Rc::new(Dashboard::new(&config));
        window.set_child(Some(&dashboard.container));

        // モニターの初期化
        let monitor = Rc::new(RefCell::new(SystemMonitor::new(config.clone())));

        // 初期データのロード
        {
            let stats = monitor.borrow_mut().fetch();
            dashboard.update(stats);
        }

        // タイマーによる定期データ更新 (update_interval_ms ごと)
        let interval = config.monitor.update_interval_ms;
        let db_clone = Rc::clone(&dashboard);
        let monitor_clone = Rc::clone(&monitor);

        glib::timeout_add_local(std::time::Duration::from_millis(interval), move || {
            let stats = monitor_clone.borrow_mut().fetch();
            db_clone.update(stats);
            glib::ControlFlow::Continue
        });

        // 表示開始
        window.present();
    });

    // 引数を無視して起動
    app.run_with_args::<&str>(&[]);
}

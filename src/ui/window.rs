use gtk4::prelude::*;
use gtk4_layer_shell::{Layer, Edge, KeyboardMode, LayerShell};
use gtk4::cairo;
use crate::config::Config;
use std::env;

pub fn create_window(app: &gtk4::Application, config: &Config) -> gtk4::ApplicationWindow {
    // グロー効果用マージン(20px)の分、全体ウィンドウサイズを大きく調整
    let window_width = config.window.width + 40;
    let window_height = config.window.height + 40;

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("glowlay")
        .default_width(window_width)
        .default_height(window_height)
        .decorated(false)
        .focusable(false)
        .resizable(false) // コンテンツ量によってウィンドウサイズが動的に伸縮するのを防ぐため、サイズを完全固定
        .build();

    // CSSのスタイルを設定しやすくするためにクラスを付与
    window.style_context().add_class("overlay-window");

    let is_wayland = env::var("WAYLAND_DISPLAY").is_ok();

    if is_wayland {
        // Wayland環境向けの設定 (gtk4-layer-shell)
        window.init_layer_shell();
        
        // 最背面（背景の上、通常ウィンドウの下）に配置
        window.set_layer(Layer::Bottom);
        
        // 一度すべてのアンカーをリセット
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Right, false);

        // 指定されたアンカー位置に基づいてピン留めとマージンを設定
        // グロー用の余白 20px を差し引いて位置を調整します
        match config.window.anchor.as_str() {
            "top-right" => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Right, true);
                window.set_margin(Edge::Top, i32::max(0, config.window.y - 20));
                window.set_margin(Edge::Right, i32::max(0, config.window.x - 20));
            }
            "bottom-left" => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
                window.set_margin(Edge::Bottom, i32::max(0, config.window.y - 20));
                window.set_margin(Edge::Left, i32::max(0, config.window.x - 20));
            }
            "bottom-right" => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Right, true);
                window.set_margin(Edge::Bottom, i32::max(0, config.window.y - 20));
                window.set_margin(Edge::Right, i32::max(0, config.window.x - 20));
            }
            _ => { // デフォルト: "top-left"
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Left, true);
                window.set_margin(Edge::Top, i32::max(0, config.window.y - 20));
                window.set_margin(Edge::Left, i32::max(0, config.window.x - 20));
            }
        }
        
        // キーボード入力を受け取らないようにする
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_namespace("glowlay");
    }

    // クリック透過の設定（X11/Wayland 双方に有効）
    // realize（実体化）された段階で、ウィンドウのGdkSurfaceに入力領域を空に設定する
    window.connect_realize(|win| {
        let surface = win.surface();
        let empty_region = cairo::Region::create();
        surface.set_input_region(&empty_region);
    });

    window
}

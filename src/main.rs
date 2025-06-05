mod config;
mod gui;

use config::{AppConfig, ColorsConfig, get_config_paths, load_config};
use eframe::NativeOptions;
use gui::RMenuApp;

fn main() -> eframe::Result<()> {
    let (colors_path, app_path) = get_config_paths().expect("Failed to get config paths");

    let colors: ColorsConfig = load_config(&colors_path);
    let app_config: AppConfig = load_config(&app_path);

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_position(egui::pos2(app_config.position.0, app_config.position.1)),
        ..Default::default()
    };

    eframe::run_native(
        "RMenu",
        options,
        Box::new(|cc| Ok(Box::new(RMenuApp::new(cc, colors, app_config)))),
    )
}

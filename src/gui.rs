use crate::config::{AppConfig, ColorsConfig};
use eframe::egui::{self, CentralPanel, Context, FontData, FontDefinitions, FontFamily, TextEdit};
use eframe::{App, CreationContext};
use std::sync::Arc;

pub struct RMenuApp {
    input_text: String,
    selected_index: usize,
    options: Vec<String>,
    colors: ColorsConfig,
    app_config: AppConfig,
}

impl RMenuApp {
    pub fn new(cc: &CreationContext<'_>, colors: ColorsConfig, app_config: AppConfig) -> Self {
        // Customize fonts if needed
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "Ubuntu Medium".to_string(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/Ubuntu-M.ttf"
            ))),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "Ubuntu Medium".to_string());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            input_text: String::new(),
            selected_index: 0,
            options: Vec::new(),
            colors,
            app_config,
        }
    }

    fn update_options(&mut self) {
        // Placeholder for filtering logic
        self.options = vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
        ]
        .into_iter()
        .filter(|opt| opt.to_lowercase().contains(&self.input_text.to_lowercase()))
        .collect();
    }
}

impl App for RMenuApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(
                (self.colors.text[0] * 255.0) as u8,
                (self.colors.text[1] * 255.0) as u8,
                (self.colors.text[2] * 255.0) as u8,
            ));
            // ui.style_mut().override_font_size = Some(self.colors.font_size);

            ui.add(
                TextEdit::singleline(&mut self.input_text)
                    .hint_text("Type to filter...")
                    .desired_width(f32::INFINITY),
            );

            if ui.button("Search").clicked() {
                self.update_options();
            }

            for (i, option) in self.options.iter().enumerate() {
                let label = if i == self.selected_index {
                    format!("> {}", option)
                } else {
                    option.clone()
                };
                if ui.button(label).clicked() {
                    self.selected_index = i;
                }
            }
        });
    }
}

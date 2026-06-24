use crate::config::{AppConfig, ColorsConfig};
use crate::desktop_entry::{self, DesktopEntry};
use eframe::egui::{self, Context, FontData, FontDefinitions, FontFamily, TextEdit};
use eframe::{App, CreationContext};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use linux_terminal_launch::{LaunchCommand, TerminalLauncher, TerminalPreference};
use std::sync::Arc;

pub struct RMenuApp {
    input_text: String,
    selected_index: usize,
    all_applications: Vec<DesktopEntry>,
    filtered_applications: Vec<DesktopEntry>,
    colors: ColorsConfig,
    #[allow(dead_code)]
    app_config: AppConfig,
    locales: Vec<String>,
    matcher: SkimMatcherV2,
}

impl RMenuApp {
    pub fn new(cc: &CreationContext<'_>, colors: ColorsConfig, app_config: AppConfig) -> Self {
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

        let locales = freedesktop_desktop_entry::get_languages_from_env();
        let all_applications = desktop_entry::get_all_applications(&locales);
        let filtered_applications = all_applications.clone();

        Self {
            input_text: String::new(),
            selected_index: 0,
            all_applications,
            filtered_applications,
            colors,
            app_config,
            locales,
            matcher: SkimMatcherV2::default(),
        }
    }

    fn filter_applications(&mut self) {
        let query = self.input_text.trim();

        if query.is_empty() {
            self.filtered_applications = self.all_applications.clone();
        } else {
            self.filtered_applications = self
                .all_applications
                .iter()
                .filter_map(|app| {
                    let name_score = app
                        .name(&self.locales)
                        .and_then(|n| self.matcher.fuzzy_match(n.as_ref(), query));
                    let generic_score = app
                        .generic_name(&self.locales)
                        .and_then(|g| self.matcher.fuzzy_match(g.as_ref(), query));
                    let comment_score = app
                        .comment(&self.locales)
                        .and_then(|c| self.matcher.fuzzy_match(c.as_ref(), query));

                    let best_score = [name_score, generic_score, comment_score]
                        .into_iter()
                        .flatten()
                        .max();

                    best_score.map(|_| app.clone())
                })
                .collect();
        }

        self.selected_index = self
            .selected_index
            .min(self.filtered_applications.len().saturating_sub(1));
    }

    fn execute_application(&self) {
        if let Some(app) = self.filtered_applications.get(self.selected_index)
            && let Some(exec) = app.exec()
        {
            if app.terminal() {
                let launch_cmd = LaunchCommand::new("sh").args(["-c", exec]);
                let mut launcher = TerminalLauncher::new()
                    .preference(TerminalPreference::Auto)
                    .launch_command(launch_cmd)
                    .detach_from_parent(true);
                if let Some(path) = app.path() {
                    launcher = launcher.working_dir(path);
                }
                let _ = launcher.spawn();
            } else {
                let exec_str = format!("{} &", exec);
                let mut cmd = std::process::Command::new("sh");
                cmd.arg("-c").arg(&exec_str);
                if let Some(path) = app.path() {
                    cmd.current_dir(path);
                }
                let _ = cmd.spawn();
            }

            std::process::exit(0);
        }
    }

    fn handle_key(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.selected_index = self.selected_index.saturating_sub(1);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown))
            && self.selected_index < self.filtered_applications.len().saturating_sub(1)
        {
            self.selected_index += 1;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.execute_application();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            std::process::exit(0);
        }
    }
}

impl App for RMenuApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_key(ctx);

        let bg_color = egui::Color32::from_rgb(
            (self.colors.background[0] * 255.0) as u8,
            (self.colors.background[1] * 255.0) as u8,
            (self.colors.background[2] * 255.0) as u8,
        );

        let text_color = egui::Color32::from_rgb(
            (self.colors.text[0] * 255.0) as u8,
            (self.colors.text[1] * 255.0) as u8,
            (self.colors.text[2] * 255.0) as u8,
        );

        let highlight_color = egui::Color32::from_rgb(
            (self.colors.highlight[0] * 255.0) as u8,
            (self.colors.highlight[1] * 255.0) as u8,
            (self.colors.highlight[2] * 255.0) as u8,
        );

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg_color))
            .show(ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(text_color);

                let input_response = ui.add(
                    TextEdit::singleline(&mut self.input_text)
                        .hint_text("Search applications...")
                        .desired_width(f32::INFINITY)
                        .text_color(text_color),
                );

                if input_response.changed() {
                    self.filter_applications();
                }

                ui.add_space(8.0);

                let scroll_area = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true);

                scroll_area.show(ui, |ui| {
                    for (i, app) in self.filtered_applications.iter().enumerate() {
                        let is_selected = i == self.selected_index;

                        if is_selected {
                            ui.style_mut().visuals.widgets.hovered.bg_fill = highlight_color;
                        }

                        let name = app.name(&self.locales).unwrap_or_default();
                        let response = ui.selectable_label(is_selected, name.as_ref());

                        if response.clicked() {
                            self.selected_index = i;
                            self.execute_application();
                        }

                        if is_selected {
                            ui.style_mut().visuals.widgets.hovered.bg_fill =
                                egui::Color32::TRANSPARENT;
                        }
                    }
                });
            });
    }
}

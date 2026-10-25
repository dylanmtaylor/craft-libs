//! Startup update prompt dialog for egui frontends.

use egui::{Align, Align2, Color32, Layout, RichText, Vec2, Window};

use crate::app::App;
use crate::release::Release;

/// What the user chose in the startup update prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupPromptResponse {
    /// Dialog is still open and waiting for user input.
    Pending,
    /// User chose to update and restart now.
    UpdateAndRestart,
    /// User chose to download manually via browser.
    Download,
    /// User dismissed the dialog for now.
    Later,
}

/// State for the startup update prompt dialog.
#[derive(Clone, Debug)]
pub struct StartupPrompt {
    pub app: &'static App,
    pub release: Release,
    pub running_version: String,
    pub is_self_updating: bool,
    pub open: bool,
    pub status_message: Option<String>,
    pub is_installing: bool,
}

impl StartupPrompt {
    /// Creates a new prompt dialog state for the given app and newly discovered release.
    pub fn new(app: &'static App, release: Release, running_version: impl Into<String>) -> Self {
        let is_self_updating = crate::self_update::is_self_updating(app);
        Self { app, release, running_version: running_version.into(), is_self_updating, open: true, status_message: None, is_installing: false }
    }

    /// Renders the modal window on the given egui context.
    pub fn show(&mut self, ctx: &egui::Context) -> StartupPromptResponse {
        if !self.open {
            return StartupPromptResponse::Later;
        }

        let mut response = StartupPromptResponse::Pending;

        // Modal dimmed background overlay
        let screen_rect = ctx.content_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("craft_update_backdrop")));
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));

        let title = format!("{} Update", self.app.name);
        let mut window_open = self.open;

        Window::new(title)
            .id(egui::Id::new("craft_update.startup_prompt"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .open(&mut window_open)
            .show(ctx, |ui| {
                ui.set_min_width(380.0);
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

                ui.vertical(|ui| {
                    ui.heading(format!("{} is available", self.release.version_label()));
                    ui.add_space(4.0);

                    let version_info = format!(
                        "A new version of {} has been released.\nCurrently running: v{}\nLatest version: {}",
                        self.app.name,
                        self.running_version,
                        self.release.version_label()
                    );
                    ui.label(version_info);

                    if let Some(msg) = &self.status_message {
                        ui.add_space(6.0);
                        ui.colored_label(Color32::from_rgb(220, 120, 50), msg);
                    }

                    if self.is_installing {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Downloading and installing update...");
                        });
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.is_installing {
                            if ui.button("Dismiss").clicked() {
                                response = StartupPromptResponse::Later;
                            }
                        } else {
                            if self.is_self_updating {
                                let update_btn = egui::Button::new(RichText::new("Update and Restart").strong());
                                if ui.add(update_btn).clicked() {
                                    response = StartupPromptResponse::UpdateAndRestart;
                                }
                            }

                            if ui.button("Download from Web").clicked() {
                                response = StartupPromptResponse::Download;
                            }

                            if ui.button("Later").clicked() {
                                response = StartupPromptResponse::Later;
                            }
                        }
                    });
                });
            });

        if !window_open {
            self.open = false;
            response = StartupPromptResponse::Later;
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suite::PHOTOCRAFT;

    #[test]
    fn creates_startup_prompt_dialog() {
        let release = Release { version: crate::version::parse("0.2.0"), tag: "v0.2.0".to_string(), ..Release::default() };
        let prompt = StartupPrompt::new(&PHOTOCRAFT, release, "0.1.0");
        assert_eq!(prompt.app.name, "PhotoCraft");
        assert_eq!(prompt.running_version, "0.1.0");
        assert!(prompt.open);
    }
}

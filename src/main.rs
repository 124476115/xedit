use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod editor;
mod file_utils;

use editor::{EditorTab, FileType, TabId};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Code Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Code Editor",
        options,
        Box::new(|cc| -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(CodeEditorApp::new(cc)))
        }),
    )
}

struct CodeEditorApp {
    tabs: Vec<EditorTab>,
    active_tab: Option<TabId>,
    next_tab_id: TabId,
    font_size: f32,
    theme: Theme,
}

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl CodeEditorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            tabs: Vec::new(),
            active_tab: None,
            next_tab_id: 0,
            font_size: 14.0,
            theme: Theme::Dark,
        };
        app.apply_theme(&cc.egui_ctx);
        app.new_tab(None);
        app
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        match self.theme {
            Theme::Dark => {
                style.visuals.dark_mode = true;
                style.visuals.panel_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x1e);
                style.visuals.window_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x1e);
                style.visuals.extreme_bg_color = egui::Color32::from_rgb(0x21, 0x25, 0x2b);
                style.visuals.faint_bg_color = egui::Color32::from_rgb(0x2d, 0x2d, 0x2d);
                style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x2d, 0x2d, 0x2d);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x3e, 0x44, 0x51);
                style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x4b, 0x52, 0x63);
                style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x5c, 0x63, 0x70);
                style.visuals.selection.bg_fill = egui::Color32::from_rgb(0x3e, 0x44, 0x51);
                style.visuals.hyperlink_color = egui::Color32::from_rgb(0x61, 0xaf, 0xef);
            }
            Theme::Light => {
                style.visuals.dark_mode = false;
            }
        }
        ctx.set_style(style);
    }

    fn new_tab(&mut self, file_path: Option<PathBuf>) {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let mut tab = EditorTab::new(id, self.font_size);
        if let Some(path) = file_path {
            if let Err(e) = tab.load_file(&path) {
                eprintln!("Failed to load file: {}", e);
            }
        }
        self.active_tab = Some(id);
        self.tabs.push(tab);
    }

    fn close_tab(&mut self, id: TabId) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            if self.tabs[pos].modified && !self.confirm_save(&self.tabs[pos]) {
                return;
            }
            self.tabs.remove(pos);
            if self.active_tab == Some(id) {
                self.active_tab = self.tabs.last().map(|t| t.id);
            }
        }
    }

    fn confirm_save(&self, _tab: &EditorTab) -> bool {
        true
    }

    fn current_tab_mut(&mut self) -> Option<&mut EditorTab> {
        let id = self.active_tab?;
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    fn current_tab(&self) -> Option<&EditorTab> {
        let id = self.active_tab?;
        self.tabs.iter().find(|t| t.id == id)
    }
}

impl eframe::App for CodeEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("新建  Ctrl+N").clicked() {
                        self.new_tab(None);
                        ui.close_menu();
                    }
                    if ui.button("打开  Ctrl+O").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("All Files", &["*"])
                            .add_filter("Markdown", &["md", "markdown"])
                            .add_filter("XML", &["xml", "xsd", "xsl"])
                            .add_filter("JSON", &["json", "jsonc"])
                            .add_filter("Env", &["env"])
                            .add_filter("Properties", &["properties", "props", "ini"])
                            .pick_file()
                        {
                            self.new_tab(Some(path));
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("保存  Ctrl+S").clicked() {
                        if let Some(tab) = self.current_tab_mut() {
                            let _ = tab.save_file();
                        }
                        ui.close_menu();
                    }
                    if ui.button("另存为  Ctrl+Shift+S").clicked() {
                        if let Some(tab) = self.current_tab_mut() {
                            let _ = tab.save_file_as();
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("关闭标签  Ctrl+W").clicked() {
                        if let Some(id) = self.active_tab {
                            self.close_tab(id);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("退出").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("编辑", |ui| {
                    if ui.button("撤销  Ctrl+Z").clicked() {
                        if let Some(tab) = self.current_tab_mut() {
                            tab.undo();
                        }
                        ui.close_menu();
                    }
                    if ui.button("重做  Ctrl+Y").clicked() {
                        if let Some(tab) = self.current_tab_mut() {
                            tab.redo();
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("视图", |ui| {
                    if ui.selectable_label(self.theme == Theme::Dark, "深色主题").clicked() {
                        self.theme = Theme::Dark;
                        ui.close_menu();
                    }
                    if ui.selectable_label(self.theme == Theme::Light, "浅色主题").clicked() {
                        self.theme = Theme::Light;
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("➕").on_hover_text("新建文件").clicked() {
                        self.new_tab(None);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.tabs.is_empty() {
                self.new_tab(None);
            }

            let mut tab_to_close: Option<TabId> = None;
            let mut new_active: Option<TabId> = None;

            egui::TopBottomPanel::top("tab_bar").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in &self.tabs {
                        let is_active = self.active_tab == Some(tab.id);
                        let mut label = tab.display_name();
                        if tab.modified {
                            label = format!("● {}", label);
                        }

                        let response = ui.selectable_label(is_active, label)
                            .on_hover_text(
                            tab.file_path
                                .as_deref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "未命名".to_string()),
                        );

                        if response.clicked() {
                            new_active = Some(tab.id);
                        }

                        if response.secondary_clicked() {
                            tab_to_close = Some(tab.id);
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("➕").clicked() {
                            self.new_tab(None);
                        }
                    });
                });
            });

            if let Some(id) = new_active {
                self.active_tab = Some(id);
            }
            if let Some(id) = tab_to_close {
                self.close_tab(id);
            }

            if let Some(tab) = self.current_tab_mut() {
                tab.show(ui, &mut self.font_size);
            }
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(tab) = self.current_tab() {
                    ui.label(format!("{}  |  {}  |  Ln {}, Col {}",
                        tab.file_type.icon(),
                        tab.file_path.as_deref().unwrap_or("未命名"),
                        tab.cursor_line,
                        tab.cursor_col
                    ));
                } else {
                    ui.label("就绪");
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("字体: {:.0}pt", self.font_size));
                });
            });
        });

        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                self.new_tab(None);
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.new_tab(Some(path));
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                if i.modifiers.shift {
                    if let Some(tab) = self.current_tab_mut() {
                        let _ = tab.save_file_as();
                    }
                } else if let Some(tab) = self.current_tab_mut() {
                    let _ = tab.save_file();
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::W) {
                if let Some(id) = self.active_tab {
                    self.close_tab(id);
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Tab) {
                if let Some(current_idx) = self.tabs.iter().position(|t| t.id == self.active_tab.unwrap_or(0)) {
                    let next = (current_idx + 1) % self.tabs.len();
                    self.active_tab = Some(self.tabs[next].id);
                }
            }
        });
    }
}
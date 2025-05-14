#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, ViewportCommand};
use eframe::icon_data;
use egui::{Color32, Margin, ScrollArea, Sense, TextEdit, vec2};
use std::env::args;
use std::fs::rename;
use std::path::PathBuf;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 420.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(true)
            .with_icon(icon_data::from_png_bytes(include_bytes!("../assets/IconLess.png")).unwrap_or_default()),
        ..Default::default()
    };
    eframe::run_native(
        "Less",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

struct MyApp {
    file_path: Option<PathBuf>,
    filename: String,
    modified: bool,
    ogcode: String,
    language: String,
    code: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let args: Vec<String> = args().collect();
        if args.len() > 1 {
            let file_path = PathBuf::from(&args[1]);
            if file_path.exists() && file_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    return Self {
                        filename: file_path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or_else(|| "Untitled")
                            .to_string(),
                        language: file_path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "".to_string()),
                        file_path: Some(file_path),
                        modified: false,
                        code: content.clone(),
                        ogcode: content,
                    };
                }
            }
        };
        print!("args: {:?}", args);
        Self {
            filename: "Untitled".to_string(),
            file_path: None,
            modified: false,
            ogcode: String::new(),
            language: "rs".into(),
            code: "fn main() {\n\
                    \tprintln!(\"Hello world!\");\n\
                    }\n"
            .into(),
        }
    }
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let title = self.file_path
        //     .as_ref()
        //     .and_then(|path| path.file_name())
        //     .and_then(|name| name.to_str())
        //     .unwrap_or("Untitled");
        if ctx.input(|i| i.key_down(egui::Key::S) && i.modifiers.ctrl && self.modified) {
            if let Some(path) = &self.file_path {
                if std::fs::write(path, &self.code).is_ok() {
                    self.ogcode = self.code.clone();
                    self.modified = false;
                }
            } else {
                if let Some(save_path) = rfd::FileDialog::new()
                    .set_title("Save File")
                    .set_file_name(&self.filename)
                    .save_file()
                {
                    if std::fs::write(&save_path, &self.code).is_ok() {
                        self.ogcode = self.code.clone();
                        self.modified = false;
                        self.filename = save_path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or_else(|| "Untitled")
                            .to_string();
                        self.language = save_path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "".to_string());
                        self.file_path = Some(save_path);
                    } else {
                        eprintln!("Error saving file");
                    }
                }
            }
        }

        let char_count = self.code.chars().count();

        custom_window_frame(
            ctx,
            &mut self.filename,
            self.modified,
            &mut self.file_path,
            |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let response = ui.add(
                        TextEdit::multiline(&mut self.code)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(10)
                            .desired_width(f32::INFINITY)
                            .background_color(Color32::from_rgb(0, 0, 0))
                            .frame(false)
                            .margin(Margin::symmetric(20, 14)),
                    );
                    if response.changed() {
                        self.modified = self.code != self.ogcode;
                    }
                });
            },
            char_count,
        );
    }
}

fn custom_window_frame(
    ctx: &egui::Context,
    title: &mut String,
    modified: bool,
    filepath: &mut Option<PathBuf>,
    add_contents: impl FnOnce(&mut egui::Ui),
    char_count: usize,
) {
    use egui::{CentralPanel, UiBuilder};

    let panel_frame = egui::Frame::new()
        .fill(Color32::from_rgb(0, 0, 0))
        .corner_radius(10)
        .outer_margin(1);

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };

        title_bar_ui(ui, title_bar_rect, title, modified, filepath);
        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect.max.y -= title_bar_height;
            rect
        }
        .shrink(4.0);
        let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
        add_contents(&mut content_ui);
        let sub_bar_rect = {
            let mut rect = app_rect;
            rect.min.y = rect.max.y - title_bar_height;
            rect
        };
        sub_bar_ui(ui, sub_bar_rect, char_count);
    });
}

fn sub_bar_ui(ui: &mut egui::Ui, sub_bar_rect: eframe::epaint::Rect, char_count: usize) {
    use egui::{Align2, FontId, Id, PointerButton, Sense, UiBuilder};
    let painter = ui.painter();

    let sub_bar_response = ui.interact(sub_bar_rect, Id::new("sub_bar"), Sense::click_and_drag());

    painter.text(
        sub_bar_rect.center(),
        Align2::CENTER_CENTER,
        char_count.to_string() + " characters",
        FontId::proportional(15.0),
        ui.style().visuals.text_color(),
    );

    if sub_bar_response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(sub_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
        },
    );
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    title_bar_rect: eframe::epaint::Rect,
    title: &mut String,
    modified: bool,
    filepath: &mut Option<PathBuf>,
) {
    use egui::{
        // Align2,
        FontId,
        Id,
        PointerButton,
        Rect, //vec2
        Sense,
        UiBuilder,
    };

    let painter = ui.painter();
    let space = painter.layout_no_wrap(
        title.to_string(),
        FontId::proportional(15.0),
        Color32::WHITE,
    );
    let text_with = space.size().x + 10.0;
    let title_bar_response = ui.interact(
        title_bar_rect,
        Id::new("title_bar"),
        Sense::click_and_drag(),
    );

    // painter.text(
    //     title_bar_rect.center(),
    //     Align2::CENTER_CENTER,
    //     title,
    //     FontId::proportional(15.0),
    //     ui.style().visuals.text_color(),
    // );

    let maxwidth: f32 = 200.0;
    let width = maxwidth.min(text_with);

    let sizerect = vec2(width, 20.0);
    let center_pos = title_bar_rect.center() - 0.5 * sizerect;

    let edit_rect = Rect::from_min_size(center_pos, sizerect);

    ui.allocate_new_ui(UiBuilder::new().max_rect(edit_rect), |ui| {
        let response = ui.add(
            TextEdit::singleline(title)
                .font(FontId::proportional(15.0))
                .text_color(if modified {
                    Color32::from_rgb(255, 0, 0)
                } else {
                    Color32::WHITE
                })
                .horizontal_align(egui::Align::Center)
                .background_color(Color32::from_rgb(0, 0, 0))
                .frame(true),
        );
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(oldpath) = filepath.take() {
                if let Some(parent) = oldpath.parent() {
                    let new_path = parent.join(title);
                    if let Err(e) = rename(&oldpath, &new_path) {
                        eprintln!("Error renaming file: {}", e);
                        *filepath = Some(oldpath);
                    } else {
                        *filepath = Some(new_path);
                    }
                }
            }
        }
    });

    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
    }

    if title_bar_response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui);
        },
    );
}

// Close Button
fn close_maximize_minimize(ui: &mut egui::Ui) {
    let close_response_size = vec2(16.0, 16.0);

    let (rect, response) = ui.allocate_exact_size(close_response_size, Sense::click());

    let painter = ui.painter();
    painter.circle_filled(rect.center(), 7.0, Color32::RED);

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

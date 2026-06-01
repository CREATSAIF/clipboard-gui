use arboard::Clipboard;
use chrono::{DateTime, Local};
use egui::{Label, ScrollArea, TextEdit};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

const MAX_HISTORY: usize = 100;
const POLL_INTERVAL_MS: u64 = 500;
const STORAGE_FILE: &str = "clipboard_history.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub char_count: usize,
    pub preview: String,
}

impl ClipboardItem {
    pub fn new(content: String) -> Self {
        let char_count = content.chars().count();
        let preview = if char_count > 200 {
            content.chars().take(200).collect::<String>() + "..."
        } else {
            content.clone()
        };
        Self {
            content,
            timestamp: Local::now(),
            char_count,
            preview,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardHistory {
    pub items: VecDeque<ClipboardItem>,
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }
}

pub struct ClipboardManager {
    clipboard: Mutex<Clipboard>,
    history: Mutex<ClipboardHistory>,
    last_content: Mutex<String>,
    storage_path: PathBuf,
}

impl ClipboardManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let storage_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clipboard-gui")
            .join(STORAGE_FILE);

        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let history = Self::load_history(&storage_path);

        Ok(Self {
            clipboard: Mutex::new(Clipboard::new()?),
            history: Mutex::new(history),
            last_content: Mutex::new(String::new()),
            storage_path,
        })
    }

    fn load_history(path: &PathBuf) -> ClipboardHistory {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(h) = serde_json::from_str::<ClipboardHistory>(&data) {
                return h;
            }
        }
        ClipboardHistory::default()
    }

    fn save_history(&self) {
        let history = self.history.lock();
        if let Ok(data) = serde_json::to_string_pretty(&*history) {
            let _ = fs::write(&self.storage_path, data);
        }
    }

    pub fn poll(&self) -> Option<ClipboardItem> {
        let mut clipboard = self.clipboard.lock();
        let mut last = self.last_content.lock();

        if let Ok(content) = clipboard.get_text() {
            if !content.is_empty() && content != *last {
                *last = content.clone();
                drop(last);

                let item = ClipboardItem::new(content);
                let mut history = self.history.lock();
                if let Some(existing) = history.items.iter().find(|i| i.content == item.content) {
                    drop(history);
                    let mut h = self.history.lock();
                    let idx = h
                        .items
                        .iter()
                        .position(|i| i.content == item.content)
                        .unwrap();
                    let mut updated = existing.clone();
                    updated.timestamp = Local::now();
                    h.items.remove(idx);
                    h.items.push_front(updated);
                } else {
                    if history.items.len() >= MAX_HISTORY {
                        history.items.pop_back();
                    }
                    history.items.push_front(item.clone());
                }
                drop(history);
                self.save_history();
                return Some(item);
            }
        }
        None
    }

    pub fn copy_to_clipboard(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = self.clipboard.lock();
        clipboard.set_text(content.to_string());
        let mut last = self.last_content.lock();
        *last = content.to_string();
        Ok(())
    }

    pub fn get_history(&self) -> Vec<ClipboardItem> {
        self.history.lock().items.iter().cloned().collect()
    }

    pub fn clear_history(&self) {
        let mut history = self.history.lock();
        history.items.clear();
        drop(history);
        self.save_history();
    }

    pub fn delete_item(&self, idx: usize) {
        let mut history = self.history.lock();
        if idx < history.items.len() {
            history.items.remove(idx);
            drop(history);
            self.save_history();
        }
    }
}

struct App {
    manager: Arc<ClipboardManager>,
    search_query: String,
    selected_index: Option<usize>,
    show_preview: Option<usize>,
    _poll_handle: Option<std::thread::JoinHandle<()>>,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = Arc::new(ClipboardManager::new()?);
        let mgr_clone = Arc::clone(&manager);

        let handle = std::thread::spawn(move || loop {
            let _ = mgr_clone.poll();
            std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        });

        Ok(Self {
            manager,
            search_query: String::new(),
            selected_index: None,
            show_preview: None,
            _poll_handle: Some(handle),
        })
    }

    fn filtered_items(&self) -> Vec<ClipboardItem> {
        let all = self.manager.get_history();
        if self.search_query.is_empty() {
            all
        } else {
            all.into_iter()
                .filter(|item| {
                    item.content
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                })
                .collect()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let items = self.filtered_items();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📋 Clipboard History");
                ui.separator();
                if ui.button("🗑 Clear All").clicked() {
                    self.manager.clear_history();
                    self.selected_index = None;
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Search:");
                TextEdit::singleline(&mut self.search_query)
                    .placeholder_text("Filter clipboard history...")
                    .ui(ui);
                if !self.search_query.is_empty() {
                    if ui.button("✕").clicked() {
                        self.search_query.clear();
                    }
                }
            });

            ui.add_space(8.0);

            if items.is_empty() {
                ui.label("No clipboard items yet. Copy something to get started!");
            } else {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (i, item) in items.iter().enumerate() {
                            let is_selected = self.selected_index == Some(i);
                            let frame = egui::Frame::default()
                                .fill(if is_selected {
                                    egui::Color32::from_rgba_unmultiplied(50, 100, 150, 40)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .inner_margin(8.0)
                                .rounding(4.0);

                            egui::Frame::fill(ui.style(), frame).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let time_str = item.timestamp.format("%H:%M:%S").to_string();
                                    ui.label(egui::RichText::new(time_str).small().weak());
                                    ui.separator();
                                    ui.label(format!("{} chars", item.char_count));
                                    ui.separator();
                                    ui.label(
                                        egui::RichText::new(item.preview.replace('\n', " "))
                                            .monospace(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("📋 Copy").clicked() {
                                                let _ =
                                                    self.manager.copy_to_clipboard(&item.content);
                                            }
                                            if ui.button("🗑").clicked() {
                                                self.manager.delete_item(i);
                                                if self.selected_index == Some(i) {
                                                    self.selected_index = None;
                                                }
                                                return;
                                            }
                                            if ui.button("👁").clicked() {
                                                self.show_preview = if self.show_preview == Some(i)
                                                {
                                                    None
                                                } else {
                                                    Some(i)
                                                };
                                            }
                                        },
                                    );
                                });

                                if self.show_preview == Some(i) {
                                    ui.add_space(4.0);
                                    ui.separator();
                                    TextEdit::multiline().frame(false).show(ui, |ui| {
                                        ui.label(item.content.clone());
                                    });
                                }
                            });

                            ui.add_space(4.0);

                            if ui
                                .input()
                                .pointer
                                .button_clicked(egui::PointerButton::Primary)
                            {
                                if ui.layout().rect.contains(ui.input().pointer.interact_pos()) {
                                    if self.selected_index == Some(i) {
                                        let _ = self.manager.copy_to_clipboard(&item.content);
                                    } else {
                                        self.selected_index = Some(i);
                                    }
                                }
                            }
                        }
                    });
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{} items", items.len())).weak());
                ui.separator();
                ui.label(egui::RichText::new("Press Enter to copy selected").weak());
            });
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Clipboard Manager")
            .with_decorations(true)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Clipboard Manager",
        options,
        Box::new(|_| Ok(Box::new(App::new()?))),
    )
    .map_err(|e| e.into())
}

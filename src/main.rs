use arboard::Clipboard;
use chrono::{DateTime, Local};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error as StdError;
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClipboardHistory {
    pub items: VecDeque<ClipboardItem>,
}

impl ClipboardHistory {
    pub fn load_from_file() -> Self {
        let path = storage_path();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save_to_file(&self) {
        let path = storage_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, data);
        }
    }
}

fn storage_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clipboard-gui")
        .join(STORAGE_FILE)
}

pub struct ClipboardManager {
    clipboard: Mutex<Clipboard>,
    history: Mutex<ClipboardHistory>,
    last_content: Mutex<String>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self, Box<dyn StdError + Send + Sync>> {
        let clipboard = Clipboard::new()?;
        let history = ClipboardHistory::load_from_file();
        Ok(Self {
            clipboard: Mutex::new(clipboard),
            history: Mutex::new(history),
            last_content: Mutex::new(String::new()),
        })
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
                if let Some(idx) = history.items.iter().position(|i| i.content == item.content) {
                    let mut updated = history.items[idx].clone();
                    updated.timestamp = Local::now();
                    history.items.remove(idx);
                    history.items.push_front(updated);
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

    pub fn copy_to_clipboard(&self, content: &str) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let mut clipboard = self.clipboard.lock();
        clipboard.set_text(content.to_string())?;
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

    fn save_history(&self) {
        let history = self.history.lock();
        history.save_to_file();
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
    fn new() -> Result<Self, Box<dyn StdError + Send + Sync>> {
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
            let q = self.search_query.to_lowercase();
            all.into_iter()
                .filter(|item| item.content.to_lowercase().contains(&q))
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
                let mut query = self.search_query.clone();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut query).hint_text("Filter clipboard history..."),
                );
                if response.changed() {
                    self.search_query = query;
                }
                if !self.search_query.is_empty() && ui.button("✕").clicked() {
                    self.search_query.clear();
                }
            });

            ui.add_space(8.0);

            if items.is_empty() {
                ui.label("No clipboard items yet. Copy something to get started!");
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (i, item) in items.iter().cloned().enumerate() {
                            let is_selected = self.selected_index == Some(i);
                            let fill_color = if is_selected {
                                egui::Color32::from_rgba_unmultiplied(50, 100, 150, 40)
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            egui::Frame::none()
                                .fill(fill_color)
                                .inner_margin(8.0)
                                .rounding(4.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let time_str =
                                            item.timestamp.format("%H:%M:%S").to_string();
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
                                                    let _ = self
                                                        .manager
                                                        .copy_to_clipboard(&item.content);
                                                }
                                                if ui.button("🗑").clicked() {
                                                    self.manager.delete_item(i);
                                                    if self.selected_index == Some(i) {
                                                        self.selected_index = None;
                                                    }
                                                }
                                                if ui.button("👁").clicked() {
                                                    self.show_preview =
                                                        if self.show_preview == Some(i) {
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
                                        ui.label(egui::RichText::new(&item.content).monospace());
                                    }
                                });

                            ui.add_space(4.0);

                            if ui.button(format!("Select #{}", i)).clicked() {
                                self.selected_index = Some(i);
                            }
                        }
                    });
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{} items", items.len())).weak());
                ui.separator();
                ui.label(egui::RichText::new("Click '📋 Copy' to copy an item").weak());
            });
        });
    }
}

fn main() -> Result<(), Box<dyn StdError + Send + Sync>> {
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
        Box::new(|_cc| Ok(Box::new(App::new()?))),
    )
    .map_err(|e| -> Box<dyn StdError + Send + Sync> {
        Box::new(std::io::Error::other(e.to_string()))
    })
}

use eframe::egui;
use regex::Regex;
use rust_todo::todo_list::{ToDoItem, ToDoList};

#[derive(Default, PartialEq)]
enum StatusFilter {
    #[default]
    All,
    Open,
    Done,
    Failed,
    Custom(String),
}

impl StatusFilter {
    fn matches(&self, status: &str) -> bool {
        match self {
            StatusFilter::All => true,
            StatusFilter::Open => status.eq_ignore_ascii_case("Open"),
            StatusFilter::Done => status.eq_ignore_ascii_case("Done"),
            StatusFilter::Failed => status.eq_ignore_ascii_case("Failed"),
            StatusFilter::Custom(s) => status.contains(s),
        }
    }
    fn label(&self) -> String {
        match self {
            StatusFilter::All => "All".into(),
            StatusFilter::Open => "Open".into(),
            StatusFilter::Done => "Done".into(),
            StatusFilter::Failed => "Failed".into(),
            StatusFilter::Custom(s) => format!("Custom({})", s),
        }
    }
}

pub struct TodoApp {
    list: ToDoList,
    path: String,
    search: String,
    use_regex: bool,
    regex_cache: Option<Regex>,
    regex_error: Option<String>,
    status_filter: StatusFilter,
    dark_mode: bool,
    filtered: Vec<(String, ToDoItem)>,
}

impl Default for TodoApp {
    fn default() -> Self {
        Self {
            list: ToDoList::new(),
            path: "todo_list.yaml".to_string(),
            search: String::new(),
            use_regex: false,
            regex_cache: None,
            regex_error: None,
            status_filter: StatusFilter::All,
            dark_mode: true,
            filtered: Vec::new(),
        }
    }
}

impl TodoApp {
    fn load_path(&mut self) {
        match ToDoList::load_from_file(&self.path) {
            Ok(list) => {
                self.list = list;
                self.filtered.clear();
                self.regex_error = None;
            }
            Err(err) => {
                self.regex_error = Some(format!("❌ Failed to load: {}", err));
            }
        }
    }

    fn save_path(&mut self) {
        if self.path.is_empty() {
            return;
        }
        if let Err(err) = self.list.save_to_file(&self.path) {
            eprintln!("Save failed: {}", err);
        }
    }

    fn run_search(&mut self) {
        self.filtered.clear();

        // update regex if needed
        if self.use_regex {
            match Regex::new(&self.search) {
                Ok(re) => {
                    self.regex_cache = Some(re);
                    self.regex_error = None;
                }
                Err(e) => {
                    self.regex_cache = None;
                    self.regex_error = Some(e.to_string());
                    return;
                }
            }
        } else {
            self.regex_cache = None;
            self.regex_error = None;
        }

        let search = self.search.clone();
        let regex_cache = self.regex_cache.as_ref();

        for (guid, item) in &self.list.items {
            let status = item.status().unwrap_or("");
            if !self.status_filter.matches(status) {
                continue;
            }

            if Self::matches_query(item, guid, regex_cache, &search) {
                self.filtered.push((guid.clone(), item.clone()));
            }
        }
    }

    fn matches_query(item: &ToDoItem, guid: &str, regex_cache: Option<&Regex>, search: &str) -> bool {
        if search.is_empty() {
            return true;
        }
        let hay = format!("{}\nGUID:{}\nYAML:{:?}", item, guid, item.to_yaml());
        if let Some(re) = regex_cache {
            re.is_match(&hay)
        } else {
            hay.to_lowercase().contains(&search.to_lowercase())
        }
    }
}

impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("🗂️ YAML ToDo Manager");
                ui.separator();
                ui.checkbox(&mut self.dark_mode, "🌗 Dark mode");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // File controls
            ui.horizontal(|ui| {
                ui.label("YAML file:");
                ui.text_edit_singleline(&mut self.path);
                if ui.button("📂 Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("YAML", &["yaml", "yml"]).pick_file() {
                        self.path = path.display().to_string();
                        self.load_path();
                    }
                }
                if ui.button("🔄 Reload").clicked() {
                    self.load_path();
                }
                if ui.button("💾 Save").clicked() {
                    self.save_path();
                }
            });

            ui.separator();

            // Search + filter
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search);
                ui.checkbox(&mut self.use_regex, "Regex");
                if ui.button("🔍 Search").clicked() {
                    self.run_search();
                }

                ui.separator();
                ui.label("Filter:");
                egui::ComboBox::from_id_source("status_filter")
                    .selected_text(self.status_filter.label())
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(matches!(self.status_filter, StatusFilter::All), "All").clicked() {
                            self.status_filter = StatusFilter::All;
                        }
                        if ui.selectable_label(matches!(self.status_filter, StatusFilter::Open), "Open").clicked() {
                            self.status_filter = StatusFilter::Open;
                        }
                        if ui.selectable_label(matches!(self.status_filter, StatusFilter::Done), "Done").clicked() {
                            self.status_filter = StatusFilter::Done;
                        }
                        if ui.selectable_label(matches!(self.status_filter, StatusFilter::Failed), "Failed").clicked() {
                            self.status_filter = StatusFilter::Failed;
                        }
                        if ui.selectable_label(matches!(self.status_filter, StatusFilter::Custom(_)), "Custom…").clicked() {
                            self.status_filter = StatusFilter::Custom(String::new());
                        }
                    });

                if let StatusFilter::Custom(ref mut s) = self.status_filter {
                    ui.text_edit_singleline(s);
                }
            });

            if let Some(err) = &self.regex_error {
                ui.colored_label(egui::Color32::YELLOW, format!("Regex error: {}", err));
            }

            ui.separator();

            // Results
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                if self.filtered.is_empty() {
                    ui.label("🔎 No matches yet — enter a term and click Search.");
                } else {
                    for (guid, item) in &mut self.filtered {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}", item));

                                if ui.button("✅ Done").clicked() {
                                    if let Some(orig) = self.list.items.get_mut(guid) {
                                        orig.mark_done();
                                    }
                                }
                                if ui.button("🔄 Reopen").clicked() {
                                    if let Some(orig) = self.list.items.get_mut(guid) {
                                        orig.reopen();
                                    }
                                }
                                if ui.button("❌ Fail").clicked() {
                                    if let Some(orig) = self.list.items.get_mut(guid) {
                                        orig.mark_failed();
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                if let Some(p) = item.get("unity_path") {
                                    ui.label(format!("Unity: {}", p));
                                    if ui.button("📋 Copy").clicked() {
                                        ui.output_mut(|o| o.copied_text = p.to_string());
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                if let Some(p) = item.get("godot_path") {
                                    ui.label(format!("Godot: {}", p));
                                    if ui.button("📋 Copy").clicked() {
                                        ui.output_mut(|o| o.copied_text = p.to_string());
                                    }
                                }
                            });

                            ui.small(format!("GUID: {}", guid));
                        });
                        ui.add_space(8.0);
                    }
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let mut native = eframe::NativeOptions::default();
    native.viewport = egui::ViewportBuilder::default()
        .with_inner_size([980.0, 720.0])
        .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../icon.png")).unwrap());
    eframe::run_native(
        "YAML ToDo Manager",
        native,
        Box::new(|_cc| Ok(Box::new(TodoApp::default()))),
    )
}

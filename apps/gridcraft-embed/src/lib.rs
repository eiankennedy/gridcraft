//! GridCraft hosted inside Workbench's side panel: the same app as the desktop build, drawn into a view
//! Workbench owns, with Workbench's look and only the sheet showing (Hands off) until the user asks for the
//! menus. Workbench (and Claude) drive it through its control channel. See `craft-embed`.
//!
//! One surface, so the app draws its active workbook itself (as the web build does) rather than through
//! the desktop's one-native-window-per-workbook `DocWindows`.

use craft_embed::{EmbedApp, egui};
use gridcraft_engine::Session;
use gridcraft_ui_egui::control::{self, Outcome};
use gridcraft_ui_egui::{Services, SheetApp};
use serde_json::{Value, json};

struct Grid(SheetApp);

impl EmbedApp for Grid {
    fn logic(&mut self, ctx: &egui::Context) {
        self.0.logic(ctx);
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(ctx, raw);
        // Files dropped on the view open as workbooks, as on the desktop.
        for f in std::mem::take(&mut raw.dropped_files) {
            if let Some(p) = f.path().to_str().map(str::to_string).filter(|p| !p.is_empty()) {
                self.0.open_path(&p);
            }
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.0.ui(ui);
    }
    fn control(&mut self, ctx: &egui::Context, request: &str) -> String {
        let req: Value = serde_json::from_str(request).unwrap_or(Value::Null);
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(json!({}));
        match control::handle(&mut self.0, ctx, method, &params) {
            Outcome::Done(v) => v.to_string(),
            Outcome::Screenshot { .. } => json!({"ok": false, "error": "Workbench takes the screenshot of the panel itself"}).to_string(),
        }
    }
}

fn make(ctx: &egui::Context) -> Box<dyn EmbedApp> {
    // GridCraft reads and writes files by path itself on the desktop: its Services has no read/write hooks
    let mut app = SheetApp::new(Session::new(), Services::default());
    if app.session.documents().is_empty() {
        app.session.new_workbook();
    }
    SheetApp::setup_context_for_language(ctx, app.ui.dark, app.ui.language);
    // Workbench's look and Hands off from the first frame; Workbench sets the mode's accent right after
    let _ = app.run("view.workbench", json!({"on": true}));
    let _ = app.run("view.handsOff", json!({"on": true}));
    Box::new(Grid(app))
}

craft_embed::export_embed!(make);

use eframe::egui;

pub struct UndoneApp {
    story_text: String,
}

impl UndoneApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            story_text: "Undone is starting up.\n\nNo scene loaded yet.".into(),
        }
    }
}

impl eframe::App for UndoneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Story text — top panel
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 80.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&self.story_text).size(16.0));
                    });

                ui.separator();

                // Action buttons — bottom row
                ui.horizontal(|ui| {
                    if ui.button("[ No actions available ]").clicked() {
                        // placeholder
                    }
                });
            });
        });
    }
}

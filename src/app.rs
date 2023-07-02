use egui::{Color32, Frame, Margin, Key};

pub struct App {
    name: String,
    age: u32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            name: "Filippo".to_owned(),
            age: 42,
        }
    }
}

impl App {
    pub fn update(&mut self, ctx: &egui::Context) {
        let frame = Frame::none()
            .fill(Color32::RED)
            .inner_margin(Margin::same(10.));

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            println!("Escape pressed, quitting");
            std::process::exit(0);
        }

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}

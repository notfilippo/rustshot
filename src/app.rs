use egui::*;

pub struct App {
    select: Option<Rect>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            select: None,
        }
    }
}

const SELECT_STROKE: Stroke = Stroke { width: 1., color: Color32::WHITE };
const SELECT_FONT: FontId = FontId {
    size: 14.0,
    family: FontFamily::Proportional,
};

impl App {
    pub fn update(&mut self, ctx: &egui::Context) {
        let frame = Frame::none()
            .fill(Color32::TRANSPARENT)
            .inner_margin(Margin::same(10.));

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            println!("Escape pressed, quitting");
            std::process::exit(0);
        }

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let (mut res, painter) = ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

            if let Some(pos) = res.interact_pointer_pos() {
                if let Some(select) = &mut self.select {
                    select.max = pos;
                } else {
                    self.select = Some(Rect::from_min_max(pos, pos));
                }
                res.mark_changed();
            } else {
                self.select = None;
            }

            if let Some(select) = self.select {
                // create render friendly rect
                let rect = Rect::from_two_pos(select.min, select.max);
                painter.rect_stroke(rect, 0., SELECT_STROKE);
                let size = rect.size();
                let text = format!("{}x{}", size.x as i32, size.y as i32);
                painter.text(rect.min, Align2::LEFT_TOP, text, SELECT_FONT, Color32::WHITE);
            }
        });
    }
}

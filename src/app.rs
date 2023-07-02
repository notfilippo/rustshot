use egui::*;

pub struct App {
    select: Option<Rect>,
    focus: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            select: None,
            focus: false,
        }
    }
}

const SELECT_STROKE: Stroke = Stroke {
    width: 1.,
    color: Color32::WHITE,
};
const SELECT_FONT: FontId = FontId {
    size: 14.0,
    family: FontFamily::Proportional,
};

const LESS_TRANSPARENT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 127);

impl App {
    pub fn update(&mut self, ctx: &egui::Context) {
        let frame = Frame::none().fill(Color32::TRANSPARENT);

        let focused = ctx.input(|i| i.focused);
        // check if focus changed and exit if focus is lost
        if self.focus != focused {
            self.focus = focused;
            if !focused {
                println!("Focus lost, quitting");
                std::process::exit(0);
            }
        }

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            println!("Escape pressed, quitting");
            std::process::exit(0);
        }

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let (mut res, painter) =
                ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

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
                let container = res.rect;
                // create render friendly rect
                let rect = Rect::from_two_pos(select.min, select.max);
                painter.rect_stroke(rect, 0., SELECT_STROKE);
                let size = rect.size();
                let text = format!("{}x{}", size.x as i32, size.y as i32);
                painter.text(
                    rect.min,
                    Align2::LEFT_TOP,
                    text,
                    SELECT_FONT,
                    Color32::WHITE,
                );
                let cover_top = Rect::from_min_max(container.min, pos2(rect.max.x, rect.min.y));
                let cover_bottom = Rect::from_min_max(pos2(rect.min.x, rect.max.y), container.max);
                let cover_left = Rect::from_min_max(
                    pos2(container.min.x, rect.min.y),
                    pos2(rect.min.x, container.max.y),
                );
                let cover_right = Rect::from_min_max(
                    pos2(rect.max.x, container.min.y),
                    pos2(container.max.x, rect.max.y),
                );
                let mut mesh = Mesh::with_texture(TextureId::default());
                mesh.add_colored_rect(cover_top, LESS_TRANSPARENT);
                mesh.add_colored_rect(cover_bottom, LESS_TRANSPARENT);
                mesh.add_colored_rect(cover_left, LESS_TRANSPARENT);
                mesh.add_colored_rect(cover_right, LESS_TRANSPARENT);
                painter.add(mesh);
            }
        });
    }
}

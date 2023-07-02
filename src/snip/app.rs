use egui::*;

pub struct App {
    selection: Option<Rect>,
    focus: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Running,
    Quit,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selection: None,
            focus: false,
        }
    }
}

impl App {
    pub fn selection(&self) -> Option<Rect> {
        return self.selection
    }
}

const SELECT_STROKE: Stroke = Stroke {
    width: 2.,
    color: Color32::RED,
};
const SELECT_FONT: FontId = FontId {
    size: 14.0,
    family: FontFamily::Proportional,
};

const LESS_TRANSPARENT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 127);

impl App {
    pub fn update(&mut self, ctx: &egui::Context) -> Status {
        let frame = Frame::none().fill(Color32::TRANSPARENT);

        let focused = ctx.input(|i| i.focused);
        // check if focus changed and exit if focus is lost
        if self.focus != focused {
            self.focus = focused;
            if !focused {
                println!("Focus lost, quitting");
                return Status::Quit;
            }
        }

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            println!("Escape pressed, quitting");
            return Status::Quit;
        }

        let response = egui::CentralPanel::default().frame(frame).show(ctx, |ui| -> Status {
            let (mut res, painter) =
                ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

            if let Some(pos) = res.interact_pointer_pos() {
                if let Some(select) = &mut self.selection {
                    select.max = pos;
                } else {
                    self.selection = Some(Rect::from_min_max(pos, pos));
                }
                res.mark_changed();
            } else {
                if self.selection.is_some() {
                    return Status::Quit;
                }
                self.selection = None;
            }

            if let Some(select) = self.selection {
                let container = res.rect;
                // create render friendly rect
                let rect = Rect::from_two_pos(select.min, select.max);
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

                let size = rect.size();
                let text = format!("{}x{}", size.x as i32, size.y as i32);
                painter.text(
                    rect.min,
                    Align2::LEFT_BOTTOM,
                    text,
                    SELECT_FONT,
                    Color32::WHITE,
                );
                painter.rect_stroke(rect, 0., SELECT_STROKE);
            }
            
            return Status::Running;
        });

        return response.inner;
    }
}

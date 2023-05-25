#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod native;

use egui::{Color32, Frame};
use egui_glow::EguiGlow;
use egui_winit::winit::{
    self,
    event::{Event, StartCause, WindowEvent},
    event_loop::ControlFlow,
    window::WindowLevel,
};
use native::{app_icon::AppTitleIconSetter, icon_data::IconData, window::Window};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
    TrayEvent, TrayIconBuilder,
};

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl MyApp {
    fn update(&mut self, ctx: &egui::Context) {
        let frame = Frame::none().fill(Color32::TRANSPARENT);

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

fn create_tray_menu() -> Menu {
    let tray_menu = Menu::new();

    let quit = MenuItem::new("Quit", true, None);
    tray_menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some(env!("CARGO_PKG_NAME").to_string()),
                copyright: Some(concat!("Copyright ", env!("CARGO_PKG_AUTHORS")).to_string()),
                comments: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &quit,
    ]);

    tray_menu
}

fn main() {
    // dumb stuff

    let mut app = MyApp::default();

    let app_name = "Rust Shot".to_string();

    let image_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png");
    let image = image::open(image_path).expect("Failed to open icon path");
    let image_data: IconData = image.into();
    let tray_img: tray_icon::icon::Icon = image_data
        .clone()
        .try_into()
        .expect("Unable to create tray icon");

    let window_icon: winit::window::Icon = image_data
        .clone()
        .try_into()
        .expect("Unable to create tray icon");

    let mut hack = AppTitleIconSetter::new(app_name.to_owned(), Some(image_data));

    // serous stuff

    let clear_color = [0.0, 0.0, 0.0, 0.0];

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();

    let window_builder = winit::window::WindowBuilder::new()
        .with_resizable(true)
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_window_icon(Some(window_icon))
        .with_inner_size(winit::dpi::LogicalSize {
            width: 300.0,
            height: 100.0,
        })
        .with_title(app_name) // Keep hidden until we've painted something. See https://github.com/emilk/egui/pull/2279
        .with_visible(false);

    #[cfg(target_os = "macos")]
    let window_builder = {
        use egui_winit::winit::platform::macos::WindowBuilderExtMacOS;
        window_builder.with_has_shadow(false)
    };

    let glutin_config_builder = glutin::config::ConfigTemplateBuilder::new()
        .prefer_hardware_accelerated(None)
        .with_alpha_size(8)
        .with_transparency(true);

    let gl_window = Window::new(window_builder, glutin_config_builder, &event_loop);
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            gl_window.get_proc_address(&s)
        })
    };

    let gl = std::sync::Arc::new(gl);

    let mut egui_glow = EguiGlow::new(&event_loop, gl.clone(), None);

    let pixels_per_point = gl_window.window().scale_factor() as f32;
    egui_glow.egui_winit.set_pixels_per_point(pixels_per_point);

    // dumb stuff

    // Since egui uses winit under the hood and doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(move || {
        let tray_menu = create_tray_menu();
        gtk::init().unwrap();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_icon(tray_img)
            .build()
            .unwrap();

        gtk::main();
    });

    #[cfg(not(target_os = "linux"))]
    let tray_menu = create_tray_menu();
    #[cfg(not(target_os = "linux"))]
    let _t = TrayIconBuilder::new()
        .with_icon(tray_img)
        .with_menu(Box::new(tray_menu))
        .build()
        .unwrap();

    gl_window.window().set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let quit = false;

            let repaint_after = egui_glow.run(gl_window.window(), |ctx| app.update(ctx));

            *control_flow = if quit {
                winit::event_loop::ControlFlow::Exit
            } else if repaint_after.is_zero() {
                gl_window.window().request_redraw();
                winit::event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(repaint_after)
            {
                winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                winit::event_loop::ControlFlow::Wait
            };

            {
                let screen_size_in_pixels: [u32; 2] = gl_window.window().inner_size().into();
                egui_glow::painter::clear(&gl, screen_size_in_pixels, clear_color);

                // draw things behind egui here

                egui_glow.paint(gl_window.window());

                // draw things on top of egui here

                gl_window.swap_buffers().unwrap();
            }
        };

        if let Ok(event) = TrayEvent::receiver().try_recv() {
            println!("tray event: {event:?}");
        }

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            Event::WindowEvent { event, .. } => {
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = ControlFlow::Exit;
                }

                if let WindowEvent::Resized(physical_size) = &event {
                    gl_window.resize(*physical_size);
                } else if let WindowEvent::ScaleFactorChanged { new_inner_size, .. } = &event {
                    gl_window.resize(**new_inner_size);
                }

                let event_response = egui_glow.on_event(&event);

                if event_response.repaint {
                    gl_window.window().request_redraw();
                }

                hack.update();
            }
            Event::LoopDestroyed => {
                egui_glow.destroy();
                *control_flow = ControlFlow::Exit;
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                gl_window.window().request_redraw();
            }

            _ => (),
        }
    });
}

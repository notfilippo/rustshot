#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod native;

use app::App;
use egui::Vec2;
use egui_glow::EguiGlow;
use egui_winit::winit::{
    self,
    event::{Event, StartCause, WindowEvent},
    event_loop::ControlFlow,
    window::WindowLevel, platform::x11::WindowExtX11, 
};
use native::{app_icon::AppTitleIconSetter, icon_data::IconData, window::OpenGLWindow};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
    TrayEvent, TrayIconBuilder,
};
use display_info::DisplayInfo;

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
    let mut app = App::default();

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

    let clear_color = [0.0, 0.0, 0.0, 0.0];

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();

    let min_position = DisplayInfo::all()
        .map(|x| {
            x.iter().fold(Vec2::ZERO, |acc, info| {
                Vec2::new(acc.x.min(info.x as f32), acc.y.min(info.y as f32))
            })
        })
        .unwrap_or_default();
    let max_position = DisplayInfo::all()
        .map(|x| {
            x.iter().fold(Vec2::ZERO, |acc, info| {
                Vec2::new(
                    acc.x.max(info.x as f32 + info.width as f32),
                    acc.y.max(info.y as f32 + info.height as f32),
                )
            })
        })
        .unwrap_or_default();
    let size = (max_position - min_position).abs();
    // let size = Vec2::new(200., 200.);

    let window_builder = winit::window::WindowBuilder::new()
        .with_resizable(true)
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_window_icon(Some(window_icon))
        .with_inner_size(winit::dpi::LogicalSize {
            width: size.x,
            height: size.y,
        })
        .with_title(app_name) // Keep hidden until we've painted something. See https://github.com/emilk/egui/pull/2279
        .with_visible(true)
        .with_active(true);

    #[cfg(target_os = "macos")]
    let window_builder = {
        use egui_winit::winit::platform::macos::WindowBuilderExtMacOS;
        window_builder.with_has_shadow(false)
    };
    
    #[cfg(target_os = "linux")]
    let window_builder = {
        use egui_winit::winit::platform::x11::{WindowBuilderExtX11, XWindowType};
        window_builder.with_override_redirect(true).with_x11_window_type(vec![XWindowType::Utility, XWindowType::Normal]) // Cannot steal focus
    };
    let glutin_config_builder = glutin::config::ConfigTemplateBuilder::new()
        .prefer_hardware_accelerated(None)
        .with_alpha_size(8)
        .with_transparency(true);

    let gl_window = OpenGLWindow::new(window_builder, glutin_config_builder, &event_loop);
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");
            gl_window.get_proc_address(&s)
        })
    };
    let gl = std::sync::Arc::new(gl);

    #[cfg(target_os = "linux")]
    unsafe {
        let window = gl_window.window();
        let window_id = window.xlib_window();
        let display_id = window.xlib_display();
        match (window_id, display_id) {
            (Some(window_id), Some(display_id)) => {
                let xlib = x11_dl::xlib::Xlib::open().expect("AAAA");
                (xlib.XSetInputFocus)(
                    display_id as _,
                    window_id,
                    x11_dl::xlib::RevertToNone,
                    x11_dl::xlib::CurrentTime,
                );
            }
            _ => {}
        }
    }

    let mut egui_glow = EguiGlow::new(&event_loop, gl.clone(), None);

    let pixels_per_point = gl_window.window().scale_factor() as f32;
    egui_glow.egui_winit.set_pixels_per_point(pixels_per_point);

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

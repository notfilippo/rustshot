pub mod app;
use std::cell::RefCell;

use crate::native::window::OpenGLWindow;
use crate::snip::app::App;
use egui::Rect;
use egui_glow::EguiGlow;
use egui_winit::winit::{
    self,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowLevel,
};

use self::app::Status;

pub fn run(event_loop: &mut EventLoop<()>) -> Option<Rect> {
    let app = RefCell::new(App::default());

    let app_name = "Rust Shot".to_string();
    let clear_color = [0.0, 0.0, 0.0, 0.0];

    let display = event_loop.primary_monitor().expect("failed to get primary monitor");
    let display_size = display.size();


    // + 1 is to avoid windows bug
    #[cfg(target_os = "windows")]
    let correction_factor = 1;

    #[cfg(not(target_os = "windows"))]
    let correction_factor = 0;
    
    let window_position = winit::dpi::LogicalPosition::new(0.0, 0.0);
    let window_size = winit::dpi::PhysicalSize::new(
        display_size.width + correction_factor,
        display_size.height + correction_factor, 
    );

    let window_builder = winit::window::WindowBuilder::new()
        .with_resizable(true)
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_position(window_position)
        .with_inner_size(window_size)
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

    let gl_window = OpenGLWindow::new(window_builder, glutin_config_builder, event_loop);
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            gl_window.get_proc_address(&s)
        })
    };

    let gl = std::sync::Arc::new(gl);

    let mut egui_glow = EguiGlow::new(event_loop, gl.clone(), None);

    let pixels_per_point = gl_window.window().scale_factor() as f32;
    egui_glow.egui_winit.set_pixels_per_point(pixels_per_point);

    gl_window.window().set_visible(true);

    event_loop.run_return(|event, _, control_flow| {
        let mut redraw = || {
            let mut status = Status::Running;

            let repaint_after = egui_glow.run(gl_window.window(), |ctx| {
                status = app.borrow_mut().update(ctx);
            });

            *control_flow = if status == Status::Quit {
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

    let app = app.borrow();
    app.selection()
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod native;
mod snip;

use egui_winit::winit;
use native::icon_data::IconData;
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

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
    let image_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png");
    let image = image::open(image_path).expect("Failed to open icon path");
    let image_data: IconData = image.into();
    let tray_img: tray_icon::icon::Icon = image_data
        .clone()
        .try_into()
        .expect("Unable to create tray icon");

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

    let mut event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();

    #[cfg(not(target_os = "linux"))]
    let tray_menu = crate::create_tray_menu();
    #[cfg(not(target_os = "linux"))]
    let _tray_icon = TrayIconBuilder::new()
        // .with_icon(tray_img)
        .with_menu(Box::new(tray_menu))
        .build()
        .unwrap();

    let rect = snip::run(&mut event_loop);
    println!("Snip: {:?}", rect)
}

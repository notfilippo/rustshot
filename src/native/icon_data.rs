#[derive(Clone)]
pub struct IconData {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl IconData {
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, String> {
        let image: image::RgbaImage = self.clone().try_into()?;
        let mut png_bytes: Vec<u8> = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageOutputFormat::Png,
            )
            .map_err(|err| err.to_string())?;
        Ok(png_bytes)
    }
}

impl From<image::DynamicImage> for IconData {
    fn from(value: image::DynamicImage) -> Self {
        let image = value.into_rgba8();
        Self {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        }
    }
}

impl TryInto<image::RgbaImage> for IconData {
    type Error = String;

    fn try_into(self) -> Result<image::RgbaImage, Self::Error> {
        image::RgbaImage::from_raw(self.width, self.height, self.rgba)
            .ok_or_else(|| "Invalid IconData".to_owned())
    }
}

impl TryInto<tray_icon::icon::Icon> for IconData {
    type Error = String;

    fn try_into(self) -> Result<tray_icon::icon::Icon, Self::Error> {
        tray_icon::icon::Icon::from_rgba(self.rgba, self.width, self.height)
            .map_err(|_| "Invalid IconData".to_owned())
    }
}

impl TryInto<egui_winit::winit::window::Icon> for IconData {
    type Error = String;

    fn try_into(self) -> Result<egui_winit::winit::window::Icon, Self::Error> {
        egui_winit::winit::window::Icon::from_rgba(self.rgba, self.width, self.height)
            .map_err(|_| "Invalid IconData".to_owned())
    }
}

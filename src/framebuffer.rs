use raylib::prelude::{Color, Image};

pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, clear_color: Color) -> Self {
        Self {
            width,
            height,
            pixels: vec![clear_color; (width * height) as usize],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let index = (y * self.width + x) as usize;
        self.pixels[index] = color;
    }

    pub fn to_image(&self) -> Image {
        let mut image = Image::gen_image_color(self.width as i32, self.height as i32, Color::BLACK);

        for y in 0..self.height {
            for x in 0..self.width {
                let index = (y * self.width + x) as usize;
                image.draw_pixel(x as i32, y as i32, self.pixels[index]);
            }
        }

        image
    }
}

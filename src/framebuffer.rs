use raylib::prelude::{Color, Image};

pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

pub struct FramebufferDifference {
    pub different_pixels: usize,
    pub maximum_channel_difference: u8,
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

    pub fn difference(&self, other: &Self) -> Result<FramebufferDifference, String> {
        if self.width != other.width || self.height != other.height {
            return Err("cannot compare framebuffers with different dimensions".to_owned());
        }

        let mut different_pixels = 0;
        let mut maximum_channel_difference = 0;
        for (left, right) in self.pixels.iter().zip(&other.pixels) {
            let channel_difference = left
                .r
                .abs_diff(right.r)
                .max(left.g.abs_diff(right.g))
                .max(left.b.abs_diff(right.b))
                .max(left.a.abs_diff(right.a));
            if channel_difference != 0 {
                different_pixels += 1;
                maximum_channel_difference = maximum_channel_difference.max(channel_difference);
            }
        }

        Ok(FramebufferDifference {
            different_pixels,
            maximum_channel_difference,
        })
    }
}

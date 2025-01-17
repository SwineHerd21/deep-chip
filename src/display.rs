use egui::{Color32, ColorImage};

/// A monochrome 64x32 display.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Display {
    /// The width of the screen in pixels.
    width: usize,
    /// The height of the screen in pixels.
    height: usize,
    /// The state of each pixel of the screen.
    pub pixels: Vec<bool>,
}

pub const DISPLAY_SCALE: usize = 10;

impl Display {
    /// 64x32 pixels. OG CHIP-8.
    #[inline]
    pub fn small() -> Display {
        let width = 64;
        let height = 32;
        Display {
            width,
            height,
            pixels: vec![false; width * height],
        }
    }

    /// 128x64 pixels. SUPER-CHIP and XO-CHIP.
    #[inline]
    pub fn big() -> Display {
        let width = 128;
        let height = 64;
        Display {
            width,
            height,
            pixels: vec![false; width * height],
        }
    }

    /// Turn off all pixels.
    #[inline]
    pub fn clear(&mut self) {
        self.pixels = vec![false; self.width * self.height];
    }

    /// Transform the display pixels into a scaled up image.
    #[inline]
    pub fn render(&self, background_color: Color32, fill_color: Color32) -> ColorImage {
        let scale = if self.width == 64 {
            DISPLAY_SCALE // small screen
        } else {
            DISPLAY_SCALE / 2 // big screen
        };
        let mut image_data = vec![background_color; self.width * scale * self.height * scale];

        for y in 0..self.height {
            for x in 0..self.width {
                if self.pixels[(x + y * self.width) as usize] {
                    for yi in 0..scale {
                        for xi in 0..scale {
                            image_data[((x * scale + xi) + ((y * scale + yi) * self.width * scale))
                                as usize] = fill_color;
                        }
                    }
                }
            }
        }

        ColorImage {
            size: [self.width * scale, self.height * scale],
            pixels: image_data,
        }
    }
}

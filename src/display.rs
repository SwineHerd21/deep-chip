use egui::{Color32, ColorImage};

/// A monochrome 64x32 display.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Display {
    /// The state of each pixel of the screen.
    pub pixels: [bool; WIDTH * HEIGHT],
}

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const DISPLAY_SCALE: usize = 10;

impl Display {
    #[inline]
    pub fn new() -> Display {
        Display {
            pixels: [false; WIDTH * HEIGHT],
        }
    }

    /// Transform the display pixels into a scaled up image.
    #[inline]
    pub fn render(&self, background_color: Color32, fill_color: Color32) -> ColorImage {
        let mut image_data = vec![background_color; WIDTH * DISPLAY_SCALE * HEIGHT * DISPLAY_SCALE];

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if self.pixels[(x + y * WIDTH) as usize] {
                    for yi in 0..DISPLAY_SCALE {
                        for xi in 0..DISPLAY_SCALE {
                            image_data[((x * DISPLAY_SCALE + xi)
                                + ((y * DISPLAY_SCALE + yi) * WIDTH * DISPLAY_SCALE))
                                as usize] = fill_color;
                        }
                    }
                }
            }
        }

        ColorImage {
            size: [WIDTH * DISPLAY_SCALE, HEIGHT * DISPLAY_SCALE],
            pixels: image_data,
        }
    }
}

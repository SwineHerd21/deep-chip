use egui::{Color32, ColorImage};

/// A monochrome 64x32 display.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Display {
    /// The state of each pixel of the screen.
    pub pixels: Vec<bool>,
}

/// The direction where to shift to screen.
pub enum ScrollDirection {
    Right,
    Left,
    Down,
}

pub const DISPLAY_SCALE: usize = 10;

impl Display {
    /// 64x32 pixels. OG CHIP-8.
    #[inline]
    pub fn small() -> Display {
        Display {
            pixels: vec![false; 64 * 32],
        }
    }

    /// 128x64 pixels. SUPER-CHIP and XO-CHIP.
    #[inline]
    pub fn big() -> Display {
        Display {
            pixels: vec![false; 128 * 64],
        }
    }

    /// Turn off all pixels.
    #[inline]
    pub fn clear(&mut self) {
        self.pixels.fill(false);
    }

    /// Scroll the screen by a certain amount of pixels.
    pub fn scroll(
        &mut self,
        direction: ScrollDirection,
        amount: usize,
        highres: bool,
        scroll_quirk: bool,
    ) {
        // Scroll quirks scrolls by half pixel
        let amount = if scroll_quirk && !highres {
            amount / 2
        } else {
            amount
        };
        let width = if highres { 128 } else { 64 };
        let height = if highres { 64 } else { 32 };

        match direction {
            ScrollDirection::Right => {
                for y in 0..height {
                    for x in (amount..width).rev() {
                        let source = x - amount + y * width;
                        let destination = x + y * width;
                        self.pixels[destination] = self.pixels[source];
                        self.pixels[source] = false;
                    }
                }
            }
            ScrollDirection::Left => {
                for y in 0..height {
                    for x in 0..(width - amount) {
                        let source = x + amount + y * width;
                        let destination = x + y * width;
                        self.pixels[destination] = self.pixels[source];
                        self.pixels[source] = false;
                    }
                }
            }
            ScrollDirection::Down => {
                for y in (amount..height).rev() {
                    for x in 0..width {
                        let source = x + (y - amount) * width;
                        let destination = x + y * width;
                        self.pixels[destination] = self.pixels[source];
                        self.pixels[source] = false;
                    }
                }
            }
        }
    }

    /// Transform the display pixels into a scaled up image.
    #[inline]
    pub fn render(
        &self,
        highres: bool,
        background_color: Color32,
        fill_color: Color32,
    ) -> ColorImage {
        let scale = if highres {
            DISPLAY_SCALE / 2 // big screen
        } else {
            DISPLAY_SCALE // small screen
        };
        let width = if highres { 128 } else { 64 };
        let height = if highres { 64 } else { 32 };

        let mut image_data = vec![background_color; width * scale * height * scale];

        for y in 0..height {
            for x in 0..width {
                if self.pixels[x + y * width] {
                    for yi in 0..scale {
                        for xi in 0..scale {
                            image_data[(x * scale + xi) + ((y * scale + yi) * width * scale)] =
                                fill_color;
                        }
                    }
                }
            }
        }

        ColorImage {
            size: [width * scale, height * scale],
            pixels: image_data,
        }
    }
}

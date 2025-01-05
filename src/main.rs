#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use deep_chip::Chip8;
use eframe::egui;
use egui::{Color32, ColorImage, TextureHandle, TextureOptions};
use gui::*;
use rodio::{
    source::{self, SignalGenerator},
    OutputStream, Sink,
};

mod gui;

fn main() {
    let chip8 = Chip8::chip8();
    let arc_chip = Arc::new(Mutex::new(chip8));

    // setup sound
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let buzz = SignalGenerator::new(
        rodio::cpal::SampleRate(48000),
        440.0,
        source::Function::Square,
    );
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.set_volume(0.05);
    sink.append(buzz);
    sink.pause();

    eframe::run_native(
        "DEEP-CHIP",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1060.0, 650.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(Emulator::new(arc_chip, sink, &&cc.egui_ctx)))
        }),
    )
    .unwrap();
}

struct Emulator {
    /// Access to the interpreter.
    interpreter: Arc<Mutex<Chip8>>,
    /// The texture to which the display is rendered.
    screen: TextureHandle,
    /// The current ROM.
    rom: Vec<u8>,
    /// The value of the path input field.
    rom_path: String,
    /// Possible ROM loading error.
    load_error: Option<std::io::Error>,
    /// Whether to show the interpreter state window.
    show_state: bool,
    /// Whether to show the keypad state window.
    show_keys: bool,
    /// Whether to show the RAM window.
    show_ram: bool,
    /// Whether to show the ROM window.
    show_rom: bool,
    /// Whether to show the display settings window.
    show_display_settings: bool,
    /// The color of disabled pixels.
    background_color: Color32,
    /// The color of enabled pixels.
    fill_color: Color32,
}

/// The duration of a single frame - the interpreter runs at 60 fps.
const FRAME_DURATION: Duration = Duration::from_millis(1000 / 60);
/// How many interpreter cycles to run in a frame.
pub const CYCLES_PER_FRAME: u32 = 500;

impl Emulator {
    fn new(interpreter: Arc<Mutex<Chip8>>, sink: Sink, ctx: &egui::Context) -> Self {
        ctx.style_mut(|style| style.override_text_style = Some(egui::TextStyle::Monospace));

        // The interpreter thread
        let clone = Arc::clone(&interpreter);
        thread::spawn(move || loop {
            let mut chip8 = clone.lock().unwrap();

            if chip8.running() {
                for _ in 0..CYCLES_PER_FRAME {
                    chip8.execute_cycle();
                }

                chip8.tick_frame();

                // play sound if enabled
                if chip8.sound_on {
                    if chip8.get_sound() > 1 {
                        if sink.is_paused() {
                            sink.play();
                        }
                    } else if !sink.is_paused() {
                        sink.pause();
                    }
                }
            }
            drop(chip8); // unlock the mutex for the gui

            // wait for the frame to end
            thread::sleep(FRAME_DURATION);
        });

        Self {
            interpreter,
            screen: ctx.load_texture(
                "screen",
                ColorImage::new([64 * 10, 32 * 10], Color32::BLACK),
                Default::default(),
            ),
            rom: vec![0],
            rom_path: String::new(),
            load_error: None,
            show_state: true,
            show_keys: true,
            show_ram: false,
            show_rom: false,
            show_display_settings: false,
            background_color: Color32::BLACK,
            fill_color: Color32::WHITE,
        }
    }
}

impl eframe::App for Emulator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut interpreter = self.interpreter.lock().unwrap();

        // read the keyboard and update the interpreter's keys
        ctx.input(|i| {
            if interpreter.is_waiting_for_key() {
                if i.key_released(egui::Key::X) {
                    interpreter.save_awaited_key(0);
                }
                if i.key_released(egui::Key::Num1) {
                    interpreter.save_awaited_key(1);
                }
                if i.key_released(egui::Key::Num2) {
                    interpreter.save_awaited_key(2);
                }
                if i.key_released(egui::Key::Num3) {
                    interpreter.save_awaited_key(3);
                }
                if i.key_released(egui::Key::Q) {
                    interpreter.save_awaited_key(4);
                }
                if i.key_released(egui::Key::W) {
                    interpreter.save_awaited_key(5);
                }
                if i.key_released(egui::Key::E) {
                    interpreter.save_awaited_key(6);
                }
                if i.key_released(egui::Key::A) {
                    interpreter.save_awaited_key(7);
                }
                if i.key_released(egui::Key::S) {
                    interpreter.save_awaited_key(8);
                }
                if i.key_released(egui::Key::D) {
                    interpreter.save_awaited_key(9);
                }
                if i.key_released(egui::Key::Z) {
                    interpreter.save_awaited_key(10);
                }
                if i.key_released(egui::Key::C) {
                    interpreter.save_awaited_key(11);
                }
                if i.key_released(egui::Key::Num4) {
                    interpreter.save_awaited_key(12);
                }
                if i.key_released(egui::Key::R) {
                    interpreter.save_awaited_key(13);
                }
                if i.key_released(egui::Key::F) {
                    interpreter.save_awaited_key(14);
                }
                if i.key_released(egui::Key::V) {
                    interpreter.save_awaited_key(15);
                }
            }

            interpreter.set_keys([
                i.key_down(egui::Key::X),    // 0
                i.key_down(egui::Key::Num1), // 1
                i.key_down(egui::Key::Num2), // 2
                i.key_down(egui::Key::Num3), // 3
                i.key_down(egui::Key::Q),    // 4
                i.key_down(egui::Key::W),    // 5
                i.key_down(egui::Key::E),    // 6
                i.key_down(egui::Key::A),    // 7
                i.key_down(egui::Key::S),    // 8
                i.key_down(egui::Key::D),    // 9
                i.key_down(egui::Key::Z),    // A
                i.key_down(egui::Key::C),    // B
                i.key_down(egui::Key::Num4), // C
                i.key_down(egui::Key::R),    // D
                i.key_down(egui::Key::F),    // E
                i.key_down(egui::Key::V),    // F
            ])
        });

        draw_menu(
            &mut interpreter,
            ctx,
            &mut self.show_state,
            &mut self.show_ram,
            &mut self.show_rom,
            &mut self.show_keys,
            &mut self.show_display_settings,
        );
        draw_display_settings(
            ctx,
            &mut self.background_color,
            &mut self.fill_color,
            &mut self.show_display_settings,
        );
        if self.show_state {
            draw_registers(&interpreter, ctx);
        }
        if self.show_ram {
            draw_ram(&interpreter, ctx);
        }
        if self.show_rom {
            draw_rom(&mut self.rom, &mut self.show_rom, ctx);
        }
        if self.show_keys {
            draw_keypad(&interpreter, ctx);
        }
        draw_controls(
            &mut interpreter,
            &mut self.rom,
            &mut self.rom_path,
            &mut self.load_error,
            ctx,
        );

        // draw the display
        egui::CentralPanel::default().show(ctx, |ui| {
            self.screen.set(
                interpreter.get_display(self.background_color, self.fill_color),
                TextureOptions::NEAREST,
            );
            ui.centered_and_justified(|ui| ui.image((self.screen.id(), self.screen.size_vec2())));
        });

        if interpreter.running() {
            ctx.request_repaint();
        }
    }
}

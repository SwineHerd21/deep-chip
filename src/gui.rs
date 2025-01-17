use std::{fs, io::Error, mem::swap};

use e_chip::{Chip8, Quirks};
use egui::{
    style::ScrollStyle, Align, Button, Color32, Frame, Grid, Id, Label, Layout, Margin, RichText,
    ScrollArea, Stroke, TextEdit, Vec2,
};

const PC_COLOR: Color32 = Color32::from_rgb(0, 100, 255);
const I_COLOR: Color32 = Color32::from_rgb(50, 130, 0);
const TEXT_COLOR: Color32 = Color32::from_gray(200);

/*
    TODO:
    - Show interpreter mode
    - Mode selection
    - Opcode breakdown?
    - Loading files with dialog
*/

#[inline]
pub fn draw_menu(
    interpreter: &mut Chip8,
    ctx: &egui::Context,
    show_rom: &mut bool,
    show_display_settings: &mut bool,
) {
    egui::TopBottomPanel::top("menu")
        .exact_height(20.0)
        .resizable(false)
        .frame(egui::Frame::default().fill(Color32::from_rgb(15, 15, 15)))
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.add_space(5.0);
                ui.menu_button("Quirks", |ui| {
                    ui.menu_button("Presets", |ui| {
                        if ui.button("CHIP-8").clicked() {
                            interpreter.quirks = Quirks::original_chip8();
                        }
                    });

                    ui.checkbox(
                        &mut interpreter.quirks.bitwise_reset_vf,
                        "Bitwise operations reset VF",
                    ).on_hover_text("If true, the 8xy1, 8xy2 and 8xy3 opcodes will set VF to 0.\nIf true, the 8xy1, 8xy2 and 8xy3 opcodes will not modify VF.");
                    ui.checkbox(
                        &mut interpreter.quirks.direct_shifting,
                        "Shift Vx directly",
                    ).on_hover_text("If true, the 8xy6 and 8xyE opcodes will set Vx to Vx >> 1.\nIf false, the 8xy6 and 8xyE opcodes will set Vx to Vy >> 1.");
                    ui.checkbox(
                        &mut interpreter.quirks.jump_to_x,
                        "Jump with offset Vx",
                    ).on_hover_text("If true, the Bnnn opcode will jump to nnn + V0.\nIf false, the Bnnn opcode will jump to nnn + Vx.");
                    ui.checkbox(
                        &mut interpreter.quirks.save_load_increment,
                        "Memory access index register increment",
                    ).on_hover_text("If true, the Fx55 and Fx65 opcodes will set I to I + x.\nIf false, the Fx55 and Fx65 opcodes will set I to I + x + 1.");
                    ui.checkbox(
                        &mut interpreter.quirks.edge_clipping,
                        "Clip sprites at edges",
                    ).on_hover_text("If true, the Dxyn opcode will clip sprites that go off the edge of the screen.\nIf false, the Dxyn opcode will wrap sprites that go off the edge of the screen around.");
                    ui.checkbox(
                        &mut interpreter.quirks.wait_for_vblank,
                        "Wait for vblank interrupt",
                    ).on_hover_text("If true, the Dxyn opcode will wait for a vblank interrupt (happens 60 times a second) before drawing.\nIf false, the Dxyn opcode will draw immediately.");
                });

                ui.menu_button("Settings", |ui| {
                    ui.checkbox(&mut interpreter.sound_on, "Sound");
                    if ui.button("Display settings").clicked() {
                        *show_display_settings = true;
                        ui.close_menu();
                    }
                    if ui.button( "Show loaded ROM").clicked() {
                        *show_rom = true;
                        ui.close_menu();
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(5.0);
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                });
            });
        });
}

#[inline]
pub fn draw_load_modal(
    interpreter: &mut Chip8,
    ctx: &egui::Context,
    show_load_modal: &mut bool,
    rom: &mut Vec<u8>,
    rom_path: &mut String,
    load_error: &mut Option<Error>,
) {
    egui::Modal::new(Id::new("Load")).show(ctx, |ui| {
        ui.heading("Load ROM");

        ui.add(TextEdit::singleline(rom_path).hint_text("Enter path..."));

        ui.horizontal(|ui| {
            if ui.button("Load program").clicked() {
                let loaded_rom = fs::read(&rom_path);
                if let Err(e) = loaded_rom {
                    *load_error = Some(e);
                } else {
                    *load_error = None;
                    *rom = loaded_rom.unwrap();

                    interpreter.reset();
                    interpreter.load_program(&rom);

                    *show_load_modal = false;
                    rom_path.clear();
                }
            }

            if ui.button("Cancel").clicked() {
                *show_load_modal = false;
                rom_path.clear();
            }
        });

        if let Some(e) = load_error {
            ui.label(format!("Could not load ROM: {e}"));
        }
    });
}

#[inline]
pub fn draw_display_settings(
    ctx: &egui::Context,
    background_color: &mut Color32,
    fill_color: &mut Color32,
    open: &mut bool,
) {
    egui::Window::new("Display settings")
        .open(open)
        .auto_sized()
        .show(ctx, |ui| {
            ui.scope_builder(egui::UiBuilder::new(), |ui| {
                Grid::new("colors")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        let mut bg = [
                            background_color.r(),
                            background_color.g(),
                            background_color.b(),
                        ];
                        ui.label("Background color");
                        ui.color_edit_button_srgb(&mut bg);
                        *background_color = Color32::from_rgb(bg[0], bg[1], bg[2]);

                        ui.end_row();
                        let mut fill = [fill_color.r(), fill_color.g(), fill_color.b()];
                        ui.label("Fill color");
                        ui.color_edit_button_srgb(&mut fill);
                        *fill_color = Color32::from_rgb(fill[0], fill[1], fill[2]);
                    });
            });

            if ui.button("Swap").clicked() {
                swap(background_color, fill_color);
            }

            ui.horizontal(|ui| {
                if ui.button("Default").clicked() {
                    *background_color = Color32::BLACK;
                    *fill_color = Color32::WHITE;
                }
                if ui.button("Octo").clicked() {
                    *background_color = Color32::from_hex("#996600").unwrap();
                    *fill_color = Color32::from_hex("#FFCC00").unwrap();
                }
                if ui.button("Matrix").clicked() {
                    *background_color = Color32::BLACK;
                    *fill_color = Color32::GREEN;
                }
            });
        });
}

#[inline]
pub fn draw_rom(rom: &mut Vec<u8>, open: &mut bool, ctx: &egui::Context) {
    egui::Window::new("ROM")
        .open(open)
        .default_size(Vec2::new(400.0, 200.0))
        .resizable(true)
        .show(ctx, |ui| {
            ui.spacing_mut().scroll = ScrollStyle::solid();
            ui.visuals_mut().override_text_color = Some(TEXT_COLOR);

            ScrollArea::vertical()
                .scroll([false, true])
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut bytes = String::new();
                        for i in 0..rom.len() {
                            bytes += &format!("{:02X} ", rom[i]);
                        }
                        ui.label(bytes);
                    });
                });
        });
}

#[inline]
pub fn draw_controls(
    interpreter: &mut Chip8,
    rom: &mut Vec<u8>,
    show_load_modal: &mut bool,
    ctx: &egui::Context,
) {
    egui::TopBottomPanel::top("control panel")
        .show_separator_line(true)
        .show(ctx, |ui| {
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!interpreter.is_running(), Button::new("Load ROM"))
                    .clicked()
                {
                    *show_load_modal = true;
                }

                if interpreter.is_running() {
                    if ui.button("Pause").clicked() {
                        interpreter.stop();
                    }
                } else {
                    if ui.button("Run").clicked() {
                        interpreter.start();
                    }
                }

                if ui
                    .add_enabled(!interpreter.is_running(), Button::new("Step cycle"))
                    .on_hover_text("Execute one instruction")
                    .clicked()
                {
                    interpreter.execute_cycle();
                    if interpreter.frame_cycle == crate::CYCLES_PER_FRAME {
                        interpreter.tick_frame();
                    }
                }
                if ui
                    .add_enabled(!interpreter.is_running(), Button::new("Step frame"))
                    .on_hover_text("Execute until this frame completes")
                    .clicked()
                {
                    for _ in interpreter.frame_cycle..crate::CYCLES_PER_FRAME {
                        interpreter.execute_cycle();
                    }
                    interpreter.tick_frame();
                }

                if ui
                    .add_enabled(!interpreter.is_running(), Button::new("Reset"))
                    .clicked()
                {
                    interpreter.reset();
                    interpreter.load_program(&rom);
                }

                ui.visuals_mut().override_text_color = Some(TEXT_COLOR);
                if !interpreter.is_running() {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(format!(
                            "Cycle: {}/{}",
                            interpreter.frame_cycle,
                            crate::CYCLES_PER_FRAME,
                        ))
                        .on_hover_text(format!(
                            "There are 60 frames per second and {} cycles per frame.",
                            crate::CYCLES_PER_FRAME
                        ));
                    });
                }
            });

            ui.add_space(2.5);
        });
}

#[inline]
pub fn draw_registers_and_keypad(interpreter: &Chip8, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("registers")
        .show_separator_line(true)
        .resizable(false)
        .default_height(100.0)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(5.0, 0.0);
            //ui.add_space(2.5);

            ui.visuals_mut().override_text_color = Some(TEXT_COLOR);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(530.0);
                    // Registers and stuff
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Program counter:");
                            ui.colored_label(
                                PC_COLOR,
                                format!("{:04X}", interpreter.get_program_counter()),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // backwards because right to left
                            ui.colored_label(I_COLOR, format!("{:04X}", interpreter.get_i()));
                            ui.label("Index (I):");
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Current opcode:");
                            ui.colored_label(
                                PC_COLOR,
                                format!("{:04X}", interpreter.get_current_opcode()),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // backwards because right to left
                            ui.colored_label(
                                Color32::YELLOW,
                                format!("{:02X}", interpreter.get_stack_pointer()),
                            );
                            ui.label("Stack pointer:");
                        });
                    });

                    ui.separator();
                    ui.scope_builder(egui::UiBuilder::new(), |ui| {
                        Grid::new("v and stack")
                            .spacing([-10.0, 1.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.add_enabled(false, Label::new(""));
                                for i in 0..16 {
                                    ui.centered_and_justified(|ui| ui.label(format!("{:X}", i)));
                                }
                                ui.end_row();

                                ui.label("V:");
                                for i in 0..16 {
                                    ui.centered_and_justified(|ui| {
                                        ui.colored_label(
                                            Color32::YELLOW,
                                            format!("{:02X}", interpreter.get_register(i)),
                                        )
                                    });
                                }
                                ui.end_row();

                                ui.label("Stack: ");
                                for i in 0..interpreter.stack_size {
                                    let stack_text =
                                        RichText::new(format!("{:03X}", interpreter.read_stack(i)))
                                            .color(Color32::YELLOW);
                                    ui.centered_and_justified(|ui| {
                                        ui.label(if i == interpreter.get_stack_pointer() as usize {
                                            stack_text.underline() // Highlight the value the stack pointer is pointing to
                                        } else {
                                            stack_text
                                        })
                                    });
                                }
                                ui.end_row();
                            });
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Delay:");
                        ui.colored_label(
                            Color32::YELLOW,
                            format!("{:02X}", interpreter.get_delay()),
                        );

                        ui.label("Sound:");
                        ui.colored_label(
                            Color32::YELLOW,
                            format!("{:02X}", interpreter.get_sound()),
                        );

                        if interpreter.is_waiting_for_key() {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(format!(
                                    "AWAITING KEY PRESS (Will save to V{:X})",
                                    interpreter.get_key_destination_register()
                                ));
                            });
                        }
                    });
                });

                ui.separator();

                // Keypad
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    ui.spacing_mut().item_spacing = Vec2::new(-10.0, -1.0);
                    ui.visuals_mut().override_text_color = Some(TEXT_COLOR);
                    Grid::new("keys").show(ui, |ui| {
                        draw_key(ui, "1", interpreter.get_key_state(1));
                        draw_key(ui, "2", interpreter.get_key_state(2));
                        draw_key(ui, "3", interpreter.get_key_state(3));
                        draw_key(ui, "C", interpreter.get_key_state(12));
                        ui.end_row();
                        draw_key(ui, "4", interpreter.get_key_state(4));
                        draw_key(ui, "5", interpreter.get_key_state(5));
                        draw_key(ui, "6", interpreter.get_key_state(6));
                        draw_key(ui, "D", interpreter.get_key_state(13));
                        ui.end_row();
                        draw_key(ui, "7", interpreter.get_key_state(7));
                        draw_key(ui, "8", interpreter.get_key_state(8));
                        draw_key(ui, "9", interpreter.get_key_state(9));
                        draw_key(ui, "E", interpreter.get_key_state(14));
                        ui.end_row();
                        draw_key(ui, "A", interpreter.get_key_state(10));
                        draw_key(ui, "0", interpreter.get_key_state(0));
                        draw_key(ui, "B", interpreter.get_key_state(11));
                        draw_key(ui, "F", interpreter.get_key_state(15));
                    });
                });
            });

            ui.add_space(2.5);
        });
}

/// Draw a single key visual.
fn draw_key(ui: &mut egui::Ui, text: &str, key: bool) {
    Frame::default()
        .inner_margin(Margin::symmetric(11.0, 8.0))
        .fill(if key { Color32::WHITE } else { Color32::BLACK })
        .stroke(Stroke::new(1.0, Color32::WHITE))
        .show(ui, |ui| {
            ui.add_enabled(
                false,
                Label::new(
                    RichText::new(text)
                        .color(if key { Color32::BLACK } else { Color32::WHITE })
                        .size(12.0),
                ),
            );
        });
}

#[inline]
pub fn draw_ram(interpreter: &Chip8, ctx: &egui::Context) {
    egui::SidePanel::right("ram")
        .show_separator_line(true)
        .default_width(195.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("RAM");
            ui.separator();
            ui.spacing_mut().scroll = ScrollStyle::solid();
            ScrollArea::vertical()
                .scroll([false, true])
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x -= 1.; // remove space around colored bytes
                        ui.visuals_mut().override_text_color = Some(TEXT_COLOR);

                        let mut bytes = String::new();
                        for i in 0..interpreter.ram_len() as u16 {
                            if i == interpreter.get_program_counter() {
                                bytes.pop(); // Remove space
                                if !bytes.is_empty() {
                                    ui.label(&bytes);
                                }
                                bytes.clear();
                            // Highlight the current instruction
                            } else if i == interpreter.get_program_counter() + 1 {
                                ui.label(
                                    RichText::new(format!(
                                        "{:02X} {:02X}",
                                        interpreter.read_byte(i - 1),
                                        interpreter.read_byte(i)
                                    ))
                                    .background_color(PC_COLOR),
                                );
                            // Highlight the place the index register is pointing to
                            } else if i == interpreter.get_i() {
                                bytes.pop(); // Remove space
                                if !bytes.is_empty() {
                                    ui.label(&bytes);
                                }
                                bytes.clear();
                                ui.label(
                                    RichText::new(format!("{:02X}", interpreter.read_byte(i)))
                                        .background_color(I_COLOR),
                                );
                            } else {
                                bytes += &format!("{:02X} ", interpreter.read_byte(i));
                            }
                        }
                        ui.label(&bytes);
                    });
                });
        });
}

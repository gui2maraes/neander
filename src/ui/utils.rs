use super::UiState;
use crate::cpu::Neander;
use egui::{Color32, Pos2, Stroke, Ui, Vec2};
use std::fmt::{Binary, Display, UpperHex};

/// What base the UI is shown in.
/// Used to format numbers in its respective
/// base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberBase {
    Dec,
    Hex,
    Bin,
}
impl NumberBase {
    pub fn fmt(self, val: impl Binary + UpperHex + Display) -> String {
        match self {
            Self::Dec => format!("{val:03}"),
            Self::Bin => format!("{val:08b}"),
            Self::Hex => format!("{val:02X}"),
        }
    }
}
pub fn menu(ui: &mut Ui, state: &mut UiState) {
    use egui::{menu, Button};

    menu::bar(ui, |ui| {
        ui.menu_button("Base", |ui| {
            if ui.button("DEC").clicked() {
                state.base = NumberBase::Dec;
                ui.close_menu();
            }
            if ui.button("HEX").clicked() {
                state.base = NumberBase::Hex;
                ui.close_menu();
            }
            if ui.button("BIN").clicked() {
                state.base = NumberBase::Bin;
                ui.close_menu();
            }
        })
    });
}
pub fn cpu_state(ui: &mut Ui, state: &UiState) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("PC: ");
            register(ui, state.base.fmt(state.cpu.pc()));
        });
        ui.horizontal(|ui| {
            ui.label("AC: ");
            register(ui, state.base.fmt(state.cpu.acc()));
        });
        ui.horizontal(|ui| {
            ui.label("STATUS: ");
            status_flag(ui, "Z: ", state.cpu.status_zero());

            status_flag(ui, "N: ", state.cpu.status_negative());
            //ui.code(self.cpu.status().to_string());
        });
    });
}

fn register(ui: &mut Ui, content: String) {
    ui.label(
        egui::RichText::new(content)
            .code()
            .color(egui::Color32::GREEN)
            .size(18.),
    );
}

fn status_flag(ui: &mut Ui, name: &str, on: bool) {
    ui.horizontal(|ui| {
        ui.label(name);
        bit(ui, Color32::GREEN, on);
    });
}

fn bit(ui: &mut Ui, color: egui::Color32, on: bool) {
    let col = if on { color } else { Color32::WHITE };
    ui.allocate_ui(Vec2::new(9., 9.), |ui| {
        let pos = ui.next_widget_position();
        ui.add_space(2.);
        ui.painter().circle(pos, 5., col, Stroke::NONE);
        ui.add_space(2.);
    });
}

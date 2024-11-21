mod utils;
use utils::*;

use crate::cpu::Neander;

pub struct UiState {
    pub base: NumberBase,
    pub cpu: Neander,
}
impl UiState {
    pub fn new() -> Self {
        Self {
            base: NumberBase::Dec,
            cpu: Neander::new(),
        }
    }
}

pub struct NeanderSim {
    state: UiState,
}

impl NeanderSim {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        _cc.egui_ctx.set_pixels_per_point(1.2);
        Self {
            state: UiState::new(),
        }
    }
}

impl eframe::App for NeanderSim {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Options").show(ctx, |ui| {
            utils::menu(ui, &mut self.state);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NEANDER");
            utils::cpu_state(ui, &self.state);
        });
    }
}

pub fn run_ui() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Neander",
        native_options,
        Box::new(|cc| Ok(Box::new(NeanderSim::new(cc)))),
    )
    .unwrap();
}

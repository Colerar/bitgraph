#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod ffmpeg;
mod history;
mod serde_ext;

use eframe::NativeOptions;

use crate::app::MainApp;

fn main() {
  tracing_subscriber::fmt::init();
  let native_options = NativeOptions {
    drag_and_drop_support: true,
    ..Default::default()
  };
  eframe::run_native(
    "Bit Graph",
    native_options,
    Box::new(|cc| Box::new(MainApp::new(cc))),
  );
}

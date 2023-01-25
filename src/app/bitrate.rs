use std::path::PathBuf;

use egui::plot::Bar;

use crate::ffmpeg::{self, ProbeData};

pub enum AnalyzeStatus {
  NotSelected,
  WaitUpdate(PathBuf),
  FfprobeFailed {
    ffmpeg: Option<ffmpeg::Error>,
    anyhow: Option<anyhow::Error>,
  },
  Success(ProbeData),
  Draw(Vec<Bar>),
}

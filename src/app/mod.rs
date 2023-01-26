mod about_window;
mod bitrate;

use std::path::PathBuf;

use egui::{
  plot::{Bar, BarChart, Legend, Plot},
  Button, Color32,
};

use crate::{
  ffmpeg::{FfProbe, Packet},
  history::History,
};

use self::{about_window::AboutWindow, bitrate::AnalyzeStatus};

pub struct MainApp {
  about: AboutWindow,
  status: AnalyzeStatus,
  ffprobe: Option<FfProbe>,
  history: History<PathBuf>,
}

impl Default for MainApp {
  fn default() -> Self {
    Self {
      about: Default::default(),
      status: AnalyzeStatus::NotSelected,
      ffprobe: match FfProbe::find() {
        Ok(ok) => Some(ok),
        Err(err) => {
          println!("Failed to find FFprobe: {err:?}");
          None
        },
      },
      history: History::with_capacity(10, 10),
    }
  }
}

impl MainApp {
  /// Called once before the first frame.
  pub fn new(_ctx: &eframe::CreationContext<'_>) -> Self {
    Default::default()
  }
}

impl eframe::App for MainApp {
  /// Called each time the UI needs repainting, which may be many times per second.
  /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
      // The top panel is often a good place for a menu bar:
      egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
          if ui.button("Open").clicked() {
            let file = rfd::FileDialog::new()
              .set_title("Select a Media File")
              .pick_file();
            if let Some(file) = file {
              self.status = AnalyzeStatus::WaitUpdate(file.clone());
              self.history.push(file);
            };
          }

          ui.menu_button("Recent...", |ui| {
            let is_empty = self.history.is_empty();
            if !is_empty {
              let home = dirs::home_dir().map(|home| home.to_string_lossy().to_string());
              ui.set_max_width(ui.available_width() * 2.);
              let mut top = None;
              for recent in self.history.read().iter() {
                let mut str = recent.to_string_lossy();
                if let Some(home) = home.as_ref() {
                  str = str.replace(home.as_str(), "~").into();
                }
                if ui.button(str.as_ref()).clicked() {
                  self.status = AnalyzeStatus::WaitUpdate(recent.clone());
                  top = Some(recent.clone());
                  break;
                };
              }
              if let Some(top) = top {
                self.history.push(top);
              }
              ui.separator();
            };

            let clear_button = Button::new("Clear");
            if ui.add_enabled(!is_empty, clear_button).clicked() {
              self.history.clear();
            }
          });

          let close_button = Button::new("Close");
          if ui
            .add_enabled(
              !matches!(self.status, AnalyzeStatus::NotSelected),
              close_button,
            )
            .clicked()
          {
            self.status = AnalyzeStatus::NotSelected;
          }

          if ui.button("Quit").clicked() {
            frame.close();
          }
        });

        if ui.button("About").clicked() {
          let about_open = &mut self.about.about_is_open;
          *about_open = !*about_open;
        };
      });
    });

    'dropped_files: {
      let dropped_files = &ctx.input().raw.dropped_files;
      if !dropped_files.is_empty() {
        let file = dropped_files.first().unwrap();
        let Some(path) = file.path.as_ref() else {
          break 'dropped_files;
        };
        if !path.exists() {
          break 'dropped_files;
        }
        if !path.is_file() {
          break 'dropped_files;
        }
        self.status = AnalyzeStatus::WaitUpdate(path.clone());
      };
    }

    egui::CentralPanel::default().show(ctx, |ui| 'center: {
      if let AnalyzeStatus::NotSelected = self.status {
        ui.heading("No file selected");
        break 'center;
      };
      if self.ffprobe.is_none() {
        ui.heading("No FFprobe found");
        break 'center;
      }

      if let AnalyzeStatus::WaitUpdate(path) = &self.status {
        ui.horizontal_wrapped(|ui| {
          ui.heading("Updating...");
          ui.spinner();
        });

        let probe = self.ffprobe.as_ref().unwrap().probe(path, None);
        let data = match probe {
          Ok(ok) => ok,
          Err(err) => {
            self.status = AnalyzeStatus::FfprobeFailed {
              ffmpeg: None,
              anyhow: Some(err),
            };
            break 'center;
          },
        };
        if let Some(err) = data.error {
          self.status = AnalyzeStatus::FfprobeFailed {
            ffmpeg: Some(err),
            anyhow: None,
          };
          break 'center;
        }

        self.status = AnalyzeStatus::Success(data);
        break 'center;
      }

      if let AnalyzeStatus::FfprobeFailed { ffmpeg, anyhow } = &self.status {
        ui.heading("Failed to probe");
        if let Some(err) = ffmpeg {
          ui.label(format!("FFprobe result: {err:#?}"));
        }
        if let Some(err) = anyhow {
          ui.label(format!("Error: {err:#?}"));
        }
        break 'center;
      }

      if let AnalyzeStatus::Success(data) = &self.status {
        let mut packets: Vec<Packet> = data
          .packets
          .clone()
          .unwrap()
          .into_iter()
          .filter(|i| i.stream_index == 0)
          .collect();
        packets.sort_unstable_by(|a, b| a.pts.cmp(&b.pts));
        let format = data.format.clone().unwrap();
        let duration = format.duration.unwrap();

        let width = 10.0;
        let mut vec = vec![0f64; f64::ceil(duration / width) as usize];
        packets.iter().for_each(|i| {
          let index = f64::floor(i.pts_time / width) as usize;
          *vec.get_mut(index).unwrap() += i.size as f64 / width / 1024.0;
        });

        let color = Color32::from_rgb(137, 130, 247);
        let vec: Vec<_> = vec
          .into_iter()
          .enumerate()
          .map(|(idx, size)| {
            Bar::new(idx as f64 * width + (width / 2.0), size)
              .width(width)
              .fill(color)
          })
          .collect();

        self.status = AnalyzeStatus::Draw(vec);

        break 'center;
      }

      if let AnalyzeStatus::Draw(vec) = &self.status {
        let chart = BarChart::new(vec.clone())
          .color(Color32::LIGHT_BLUE)
          .name("KiB/s");

        Plot::new("BitRate")
          .legend(Legend::default())
          .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
          });

        break 'center;
      }
    });

    self.about.ui(ctx);
  }
}

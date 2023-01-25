mod about_window;

use std::path::PathBuf;

use egui::Button;

use crate::history::History;

use self::about_window::AboutWindow;

pub struct MainApp {
  about: AboutWindow,
  current: Option<PathBuf>,
  history: History<PathBuf>,
}

impl Default for MainApp {
  fn default() -> Self {
    Self {
      about: Default::default(),
      current: None,
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
            let work_dir = std::env::current_dir().unwrap();
            let file = rfd::FileDialog::new()
              .set_directory(work_dir)
              .set_title("Select a Media File")
              .pick_file();
            self.current = file.clone();
            if let Some(file) = file {
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
                  self.current = Some(recent.clone());
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

          if ui.button("Close").clicked() {
            self.current = None;
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

    egui::CentralPanel::default().show(ctx, |ui| 'center: {
      let Some(current) = self.current.as_ref() else {
        ui.heading("No file selected");
        break 'center;
      };
      ui.heading(format!("TODO, current: {}", current.to_string_lossy()));
    });

    self.about.ui(ctx);
  }
}

use egui::{
  special_emojis::{GITHUB, OS_APPLE, OS_LINUX, OS_WINDOWS},
  Context,
};

#[derive(Default)]
pub struct AboutWindow {
  about: About,
  pub about_is_open: bool,
}

impl AboutWindow {
  pub fn ui(&mut self, ctx: &Context) {
    self.about.ui(&mut self.about_is_open, ctx);
  }
}

#[derive(Default)]
pub struct About;

impl About {
  fn ui(&mut self, open: &mut bool, ctx: &Context) {
    const SEP_SPACE: f32 = 5.0;
    egui::Window::new("About")
      .open(open)
      .resizable(false)
      .show(ctx, |ui| {
        ui.heading(concat!("BitGraph v", env!("CARGO_PKG_VERSION")));
        ui.label(format!(
          "BitGraph is a media bitrate analyzer based on FFprobe,\
           written in Rust, with cross-platform support. \
           {OS_APPLE}{OS_LINUX}{OS_WINDOWS}",
        ));

        ui.add_space(SEP_SPACE);

        ui.heading("License");
        ui.horizontal_wrapped(|ui| {
          ui.spacing_mut().item_spacing.x = 0.0;
          ui.label("Distributed under the ");
          ui.hyperlink_to(
            "MIT license",
            "https://github.com/Colerar/bitgraph/blob/main/LICENSE",
          );
          ui.label(".");
        });

        ui.add_space(SEP_SPACE);

        ui.heading("Links");
        ui.hyperlink_to(
          format!("{GITHUB} Bit Graph on GitHub"),
          "https://github.com/Colerar/bitgraph",
        );
      });
  }
}

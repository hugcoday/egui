use crate::{app, demos, History, Ui};

// ----------------------------------------------------------------------------

/// How often we repaint the demo app by default
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunMode {
    /// This is the default for the demo.
    ///
    /// If this is selected, Egui is only updated if are input events
    /// (like mouse movements) or there are some animations in the GUI.
    ///
    /// Reactive mode saves CPU.
    ///
    /// The downside is that the UI can become out-of-date if something it is supposed to monitor changes.
    /// For instance, a GUI for a thermostat need to repaint each time the temperature changes.
    /// To ensure the UI is up to date you need to call `egui::Context::request_repaint()` each
    /// time such an event happens. You can also chose to call `request_repaint()` once every second
    /// or after every single frame - this is called `Continuous` mode,
    /// and for games and interactive tools that need repainting every frame anyway, this should be the default.
    Reactive,

    /// This will call `egui::Context::request_repaint()` at the end of each frame
    /// to request the backend to repaint as soon as possible.
    ///
    /// On most platforms this will mean that Egui will run at the display refresh rate of e.g. 60 Hz.
    ///
    /// For this demo it is not any reason to do so except to
    /// demonstrate how quickly Egui runs.
    ///
    /// For games or other interactive apps, this is probably what you want to do.
    /// It will guarantee that Egui is always up-to-date.
    Continuous,
}

/// Default for demo is Reactive since
/// 1) We want to use minimal CPU
/// 2) There are no external events that could invalidate the UI
///    so there are no events to miss.
impl Default for RunMode {
    fn default() -> Self {
        RunMode::Reactive
    }
}

// ----------------------------------------------------------------------------

struct FrameHistory {
    frame_times: History<f32>,
}

impl Default for FrameHistory {
    fn default() -> Self {
        let max_age: f64 = 1.0;
        Self {
            frame_times: History::from_max_len_age((max_age * 300.0).round() as usize, max_age),
        }
    }
}

impl FrameHistory {
    pub fn on_new_frame(&mut self, now: f64, previus_frame_time: Option<f32>) {
        let previus_frame_time = previus_frame_time.unwrap_or_default();
        if let Some(latest) = self.frame_times.latest_mut() {
            *latest = previus_frame_time; // rewrite history now that we know
        }
        self.frame_times.add(now, previus_frame_time); // projected
    }

    fn fps(&self) -> f32 {
        1.0 / self.frame_times.mean_time_interval().unwrap_or_default()
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.label(format!(
            "Total frames painted: {}",
            self.frame_times.total_count()
        ));

        ui.label(format!(
            "Mean CPU usage per frame: {:.2} ms / frame",
            1e3 * self.frame_times.average().unwrap_or_default()
        ))
        .on_hover_text(
            "Includes Egui layout and tesselation time.\n\
            Does not include GPU usage, nor overhead for sending data to GPU.",
        );

        crate::CollapsingHeader::new("CPU usage history")
            .default_open(false)
            .show(ui, |ui| {
                self.graph(ui);
            });
    }

    fn graph(&mut self, ui: &mut Ui) {
        use crate::*;

        let graph_top_cpu_usage = 0.010;
        ui.label("Egui CPU usage history");

        let history = &self.frame_times;

        // TODO: we should not use `slider_width` as default graph width.
        let height = ui.style().spacing.slider_width;
        let rect = ui.allocate_space(vec2(ui.available_finite().width(), height));
        let style = ui.style().noninteractive();

        let mut cmds = vec![PaintCmd::Rect {
            rect,
            corner_radius: style.corner_radius,
            fill: ui.style().visuals.dark_bg_color,
            stroke: ui.style().noninteractive().bg_stroke,
        }];

        let rect = rect.shrink(4.0);
        let line_stroke = Stroke::new(1.0, Srgba::additive_luminance(128));

        if let Some(mouse_pos) = ui.input().mouse.pos {
            if rect.contains(mouse_pos) {
                let y = mouse_pos.y;
                cmds.push(PaintCmd::line_segment(
                    [pos2(rect.left(), y), pos2(rect.right(), y)],
                    line_stroke,
                ));
                let cpu_usage = remap(y, rect.bottom_up_range(), 0.0..=graph_top_cpu_usage);
                let text = format!("{:.1} ms", 1e3 * cpu_usage);
                cmds.push(PaintCmd::text(
                    ui.fonts(),
                    pos2(rect.left(), y),
                    align::LEFT_BOTTOM,
                    text,
                    TextStyle::Monospace,
                    color::WHITE,
                ));
            }
        }

        let circle_color = Srgba::additive_luminance(196);
        let radius = 2.0;
        let right_side_time = ui.input().time; // Time at right side of screen

        for (time, cpu_usage) in history.iter() {
            let age = (right_side_time - time) as f32;
            let x = remap(age, history.max_age()..=0.0, rect.x_range());
            let y = remap_clamp(cpu_usage, 0.0..=graph_top_cpu_usage, rect.bottom_up_range());

            cmds.push(PaintCmd::line_segment(
                [pos2(x, rect.bottom()), pos2(x, y)],
                line_stroke,
            ));

            if cpu_usage < graph_top_cpu_usage {
                cmds.push(PaintCmd::circle_filled(pos2(x, y), radius, circle_color));
            }
        }

        ui.painter().extend(cmds);
    }
}

// ----------------------------------------------------------------------------

/// Demonstrates how to make an app using Egui.
///
/// Implements `egui::app::App` so it can be used with
/// [`egui_glium`](https://crates.io/crates/egui_glium) and [`egui_web`](https://crates.io/crates/egui_web).
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct DemoApp {
    demo_windows: demos::DemoWindows,

    #[cfg_attr(feature = "serde", serde(skip))] // go back to `Reactive` mode each time we start
    run_mode: RunMode,

    /// current slider value for current gui scale (backend demo only)
    pixels_per_point: Option<f32>,

    #[cfg_attr(feature = "serde", serde(skip))]
    frame_history: FrameHistory,
}

impl DemoApp {
    fn backend_ui(&mut self, ui: &mut Ui, info: &app::BackendInfo) -> app::AppOutput {
        self.frame_history
            .on_new_frame(ui.input().time, info.cpu_usage);

        let is_web = info.web_info.is_some();

        if is_web {
            ui.label("Egui is an immediate mode GUI written in Rust, compiled to WebAssembly, rendered with WebGL.");
            ui.label(
                "Everything you see is rendered as textured triangles. There is no DOM. There are no HTML elements. \
                This is not JavaScript. This is Rust, running at 60 FPS. This is the web page, reinvented with game tech.");
            ui.label("This is also work in progress, and not ready for production... yet :)");
            ui.horizontal(|ui| {
                ui.label("Project home page:");
                ui.hyperlink("https://github.com/emilk/egui");
            });
            ui.separator();
        }

        self.run_mode_ui(ui);

        ui.separator();

        self.frame_history.ui(ui);

        ui.separator();

        let mut output = app::AppOutput::default();
        output.pixels_per_point = self.pixels_per_point_ui(ui, info);

        if !is_web {
            ui.separator();
            output.quit |= ui.button("Quit").clicked;
        }

        output
    }

    fn pixels_per_point_ui(&mut self, ui: &mut Ui, info: &app::BackendInfo) -> Option<f32> {
        self.pixels_per_point = self
            .pixels_per_point
            .or(info.native_pixels_per_point)
            .or_else(|| Some(ui.ctx().pixels_per_point()));
        if let Some(pixels_per_point) = &mut self.pixels_per_point {
            ui.add(
                crate::Slider::f32(pixels_per_point, 0.5..=5.0)
                    .logarithmic(true)
                    .text("Scale (physical pixels per point)"),
            );
            if let Some(native_pixels_per_point) = info.native_pixels_per_point {
                if ui
                    .button(format!(
                        "Reset scale to native value ({:.1})",
                        native_pixels_per_point
                    ))
                    .clicked
                {
                    *pixels_per_point = native_pixels_per_point;
                }
            }
            if !ui.ctx().is_using_mouse() {
                // We wait until mouse release to activate:
                return Some(*pixels_per_point);
            }
        }
        None
    }

    fn run_mode_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let run_mode = &mut self.run_mode;
            ui.label("Run mode:");
            ui.radio_value(run_mode, RunMode::Continuous, "Continuous")
                .on_hover_text("Repaint everything each frame");
            ui.radio_value(run_mode, RunMode::Reactive, "Reactive")
                .on_hover_text("Repaint when there are animations or input (e.g. mouse movement)");
        });

        if self.run_mode == RunMode::Continuous {
            ui.label(format!(
                "Repainting the UI each frame. FPS: {:.1}",
                self.frame_history.fps()
            ));
        } else {
            ui.label("Only running UI code when there are animations or input");
        }
    }
}

impl app::App for DemoApp {
    fn ui(
        &mut self,
        ui: &mut Ui,
        info: &app::BackendInfo,
        tex_allocator: Option<&mut dyn app::TextureAllocator>,
    ) -> app::AppOutput {
        let mut output = app::AppOutput::default();

        crate::Window::new("Backend")
            .min_width(360.0)
            .scroll(false)
            .show(ui.ctx(), |ui| {
                output = self.backend_ui(ui, info);
            });

        let web_location_hash = info
            .web_info
            .as_ref()
            .map(|info| info.web_location_hash.clone())
            .unwrap_or_default();
        let link = if web_location_hash == "clock" {
            Some(demos::DemoLink::Clock)
        } else {
            None
        };

        let demo_environment = demos::DemoEnvironment {
            seconds_since_midnight: info.seconds_since_midnight,
            link,
        };

        self.demo_windows.ui(ui, &demo_environment, tex_allocator);

        if self.run_mode == RunMode::Continuous {
            // Tell the backend to repaint as soon as possible
            ui.ctx().request_repaint();
        }

        output
    }

    #[cfg(feature = "serde_json")]
    fn on_exit(&mut self, storage: &mut dyn app::Storage) {
        app::set_value(storage, app::APP_KEY, self);
    }
}

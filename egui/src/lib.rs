//! Egui core library
//!
//! To get started with Egui, you can use one of the available integrations
//! such as [`egui_web`](https://crates.io/crates/egui_web) or  [`egui_glium`](https://crates.io/crates/egui_glium).
//!
//! To write your own integration for Egui you need to do this:
//!
//! ``` ignore
//! let mut egui_ctx = egui::Context::new();
//!
//! // Game loop:
//! loop {
//!     let raw_input: egui::RawInput = my_integration.gather_input();
//!     let mut ui = egui_ctx.begin_frame(raw_input);
//!     my_app.ui(&mut ui); // add windows and widgets to `ui` here
//!     let (output, paint_jobs) = egui_ctx.end_frame();
//!     my_integration.paint(paint_jobs);
//!     my_integration.set_cursor_icon(output.cursor_icon);
//!     // Also see `egui::Output` for more
//! }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::imprecise_flops,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::pub_enum_variant_names,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::todo,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]

pub mod align;
mod animation_manager;
pub mod app;
pub(crate) mod cache;
pub mod containers;
mod context;
pub mod demos;
mod history;
mod id;
mod input;
mod introspection;
mod layers;
mod layout;
pub mod math;
mod memory;
pub mod menu;
pub mod mutex;
pub mod paint;
mod painter;
mod style;
mod types;
mod ui;
pub mod widgets;

pub use {
    align::Align,
    containers::*,
    context::Context,
    demos::DemoApp,
    history::History,
    id::Id,
    input::*,
    layers::*,
    layout::*,
    math::*,
    memory::Memory,
    paint::{color, PaintCmd, PaintJobs, Rgba, Srgba, Stroke, TextStyle, Texture, TextureId},
    painter::Painter,
    style::Style,
    types::*,
    ui::Ui,
    widgets::*,
};

#[test]
pub fn text_egui_e2e() {
    let mut demo_windows = crate::demos::DemoWindows::default();
    let mut ctx = crate::Context::new();
    let raw_input = crate::RawInput {
        screen_size: crate::vec2(1280.0, 1024.0),
        ..Default::default()
    };

    const NUM_FRAMES: usize = 5;
    for _ in 0..NUM_FRAMES {
        let mut ui = ctx.begin_frame(raw_input.clone());
        demo_windows.ui(&mut ui, &Default::default(), None);
        let (_output, paint_jobs) = ctx.end_frame();
        assert!(!paint_jobs.is_empty());
    }
}

#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::collapsible_if
)]

mod app;
mod audio;
mod battle;
mod board;
mod economy;
mod net;
mod stages;
mod synergies;
mod types;
mod ui;
mod units;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    app::run();
}

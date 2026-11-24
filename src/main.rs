//! Binary entry point. The workbench itself lives in the library crate so the
//! same code can be mounted by the headless UI tests in `tests/`.

fn main() {
    gpui_kit_demo::run();
}

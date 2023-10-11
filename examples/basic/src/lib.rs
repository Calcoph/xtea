use std::ptr;

use imgui::{Ui, sys::{ImGuiMemFreeFunc, ImGuiMemAllocFunc}};
use xtealib::ImguiLogger;

static mut STATE: State = State::new();

fn mut_state() -> &'static mut State {
    unsafe{&mut STATE}
}

fn state() -> &'static State {
    unsafe{&STATE}
}

struct State {
    show_window: bool
}

impl State {
    const fn new() -> State {
        State {
            show_window: true
        }
    }
}

#[no_mangle]
pub fn init_logger(logger: &'static ImguiLogger) {
    logger.init().unwrap()
}

#[no_mangle]
pub fn init_imgui(ctx: *mut imgui::sys::ImGuiContext, malloc: ImGuiMemAllocFunc, free: ImGuiMemFreeFunc) {
    unsafe {imgui::sys::igSetCurrentContext(ctx)}
    unsafe {imgui::sys::igSetAllocatorFunctions(malloc, free, ptr::null_mut())}
}

#[no_mangle]
pub fn init_plugin() {

}

#[no_mangle]
pub fn build_ui(ui: &Ui) {
    if state().show_window {
        ui.window("basic-example").focus_on_appearing(false).build(|| {
            ui.text("This is a window")
        });
    }
}

#[no_mangle]
pub fn get_name() -> String {
    String::from("Example (basic) Plugin")
}

#[no_mangle]
pub fn view_submenu(ui: &Ui) {
    ui.checkbox("Basic Example Window", &mut mut_state().show_window);
}

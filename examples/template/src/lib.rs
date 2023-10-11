use std::ptr;

use imgui::{Ui, sys::{ImGuiMemAllocFunc, ImGuiMemFreeFunc}};
use xtealib::ImguiLogger;

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

}

#[no_mangle]
pub fn get_name() -> String {
    String::from("My Plugin Name")
}

#[no_mangle]
pub fn view_submenu(ui: &Ui) {

}

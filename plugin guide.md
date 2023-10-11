# UX guidelines
When creating windows:
* `.focus_on_appearing(false)` is required, otherwise the "View" menu will disappear each time that window is toggled.

# Mandatory functions

A plugin **must** contain all of these public functions
* [init_imgui](#init_imgui)
* [init_plugin](#init_plugin)
* [build_ui](#build_ui)
* [get_name](#get_name)
* [init_logger](#init_logger)
* [view_submenu](#view_submenu)

## init_logger
```rust
#[no_mangle]
pub fn init_logger(logger: &'static ImguiLogger) {
    logger.init().unwrap()
}
```

## init_imgui
```rust
#[no_mangle]
pub fn init(ctx: *mut imgui::sys::ImGuiContext, malloc: ImGuiMemAllocFunc, free: ImGuiMemFreeFunc) {
    unsafe {imgui::sys::igSetCurrentContext(ctx)}
    unsafe {imgui::sys::igSetAllocatorFunctions(malloc, free, ptr::null_mut())}
}
```

## init_plugin
```rust
#[no_mangle]
pub fn init_plugin() {
    // your code here
}
```

## build_ui
```rust
#[no_mangle]
pub fn build_ui(ui: &Ui) {
    // your code here
}
```

## get_name
Must return a unique plugin name.
```rust
#[no_mangle]
pub fn get_name() -> String {
    String::from("Example Plugin")
}
```

## view_submenu
```rust
#[no_mangle]
pub fn view_submenu(ui: &Ui) {
    // your code here
}
```

# Plugin template
```rust
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
```

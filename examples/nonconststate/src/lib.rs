use std::{sync::{Mutex, MutexGuard}, ptr, collections::HashMap};

use imgui::{Ui, sys::{ImGuiMemFreeFunc, ImGuiMemAllocFunc}};
use xtealib::ImguiLogger;
use once_cell::sync::Lazy;
static mut STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::new()));

fn mut_state() -> &'static mut State {
    unsafe{STATE.get_mut().unwrap()}
}

fn state() -> MutexGuard<'static, State> {
    unsafe{STATE.lock().unwrap()}
}

struct State {
    show_window: bool,
    map: HashMap<String, String>
}

impl State {
    fn new() -> State {
        let mut map = HashMap::new();
        map.insert("Key1".into(), "Value1".into());
        map.insert("Key2".into(), "Value2".into());
        map.insert("Key3".into(), "Value3".into());
        map.insert("Key4".into(), "Value4".into());

        State {
            show_window: true,
            map
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
    let state = state();
    if state.show_window {
        ui.window("nonconststate-example").focus_on_appearing(false).build(|| {
            for (key, val) in state.map.iter() {
                ui.label_text(key, val)
            }
        });
    }
}

#[no_mangle]
pub fn get_name() -> String {
    String::from("Example (nonconststate) Plugin")
}

#[no_mangle]
pub fn view_submenu(ui: &Ui) {
    ui.checkbox("Non Const State Example Window", &mut mut_state().show_window);
}

use std::{collections::HashMap, path::{Path, PathBuf}};

use imgui::Ui;
use log::error;

use crate::{IMGUI_LOGGER, Plugins};

pub struct UiState {
    logs: HashMap<String, Vec<String>>,
    view_logs: bool,
    view_style_editor: bool,
    view_plugin_manager: bool,
}

impl UiState {
    pub fn new() -> UiState {
        UiState {
            logs: HashMap::new(),
            view_logs: true,
            view_style_editor: false,
            view_plugin_manager: false,
        }
    }
}

pub(crate) fn create_ui(ui: &Ui, state: &mut UiState, plugins: &mut Plugins) {
    ui.dockspace_over_main_viewport();

    create_menu(ui, state, plugins);
    if state.view_logs {
        logger_windw(ui, state);
    }
    if state.view_style_editor {
        style_editor(ui, state);
    }
    if state.view_plugin_manager {
        plugin_manager(ui, state, plugins);
    }

    for plugin in plugins.ui_build_iter() {
        plugin.build_ui(ui)
    }
}

fn style_editor(ui: &Ui, state: &mut UiState) {
    ui.show_default_style_editor()
}

fn plugin_manager(ui: &Ui, state: &mut UiState, plugins: &mut Plugins) {
    ui.window("Plugin Manager").focus_on_appearing(false).build(|| {
        if ui.button("Reload All Plugins") {
            plugins.reload_all_plugins();
        }

        // to avoid borrowing plugins
        let mut tmp_vec = vec![];
        std::mem::swap(&mut plugins.all_plugins, &mut tmp_vec);

        for (plugin_name, dll_path) in tmp_vec.iter() {
            let active = plugins.loaded_plugins.get(plugin_name).is_some();
            if ui.checkbox(plugin_name, &mut (active.clone())) {
                if active {
                    plugins.unload(plugin_name)
                } else {
                    plugins.activate(dll_path)
                }
            }
        }
        std::mem::swap(&mut plugins.all_plugins, &mut tmp_vec);
    });
}

fn create_menu(ui: &Ui, state: &mut UiState, plugins: &Plugins) {
    ui.main_menu_bar(|| {
        if ui.menu_item("View") {
            ui.open_popup("view_menu_item");
        };
        ui.popup("view_menu_item", || {
            ui.checkbox("Logs", &mut state.view_logs);
            ui.checkbox("Style Editor", &mut state.view_style_editor);
            ui.checkbox("Plugin Manager", &mut state.view_plugin_manager);
            for plugin in plugins.view_submenu_iter() {
                plugin.view_submenu(ui)
            }
        });
    });
}

fn logger_windw(ui: &Ui, state: &mut UiState) {
    let messages = IMGUI_LOGGER.clear();
    for (key, mut val) in messages {
        match state.logs.get_mut(&key) {
            Some(vec) => vec.append(&mut val),
            None => {state.logs.insert(key, val);},
        }
    }
    let mut save_all_logs = false;
    ui.window("Logs").focus_on_appearing(false).build(|| {
        if let Some(_) = ui.tab_bar("logs") {
            for (tab, messages) in state.logs.iter() {
                if let Some(_) = ui.tab_item(tab) {
                    if ui.button("Save Log") {
                        let content = messages.iter().fold(String::new(), |old, new| format!("{old}\n{new}"));
                        let log_path = PathBuf::from(format!("logs/{tab}.txt"));
                        if !log_path.parent().unwrap().exists() {
                            // logs folder doesn't exist
                            if let Err(e) = std::fs::create_dir("logs") {
                                error!("Could not create logs directory\n\t{e}")    
                            }
                        }
                        if let Err(e) = std::fs::write(log_path, content) {
                            error!("Could not save log {tab}.txt\n\t{e}")
                        }
                    }
                    if ui.is_item_hovered() {
                        ui.tooltip_text(format!("Saves this log to logs/{tab}.txt"))
                    }
                    if ui.button("Save All logs") {
                        save_all_logs = true
                    }
                    if ui.is_item_hovered() {
                        ui.tooltip_text(format!("Saves all the logs to logs/<name of plugin>.txt"))
                    }
                    for message in messages {
                        ui.text(message)
                    }
                }
            }
        };
    });

    if save_all_logs {
        for (tab, messages) in state.logs.iter() {
            let content = messages.iter().fold(String::new(), |old, new| format!("{old}\n{new}"));
            if let Err(e) = std::fs::write(format!("logs/{tab}.txt"), content) {
                error!("Could not save log {tab}.txt\n\t{e}")
            }
        }
    }
}

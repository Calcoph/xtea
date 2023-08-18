use imgui::Ui;

pub fn style_editor_window(ui: &Ui) {
    let mut style = ui.clone_style();
    ui.show_style_editor(&mut style);
}

use super::*;

#[test]
fn escape_closes_project_inspection_before_leaving_the_workspace() {
    let ctx = egui::Context::default();
    let mut shell = super::super::StudioShell {
        open: true,
        ..Default::default()
    };
    shell.projects.open = true;
    let input = egui::RawInput {
        events: vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = crate::presentation::test_support::run(&ctx, input, |ui| {
        shell.handle_escape(ui.ctx());
    });
    assert!(!shell.projects.open);
    assert!(shell.open);
}

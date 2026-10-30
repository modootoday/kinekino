use super::*;

#[test]
fn disabled_filters_reject_clicks_and_keyboard_keeps_original_indices() {
    let ctx = egui::Context::default();
    crate::presentation::layout::configure(&ctx);
    let filters = [
        Filter {
            label: "All",
            available: true,
        },
        Filter {
            label: "Running",
            available: false,
        },
        Filter {
            label: "Failed",
            available: true,
        },
    ];
    let mut selected = Some(0);
    let mut responses = Vec::new();
    let mut frame = |events| {
        let output = crate::presentation::test_support::run(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(240., 300.),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                responses = show(ui, "filters", &filters, &mut selected, "Unavailable");
            },
        );
        (output, responses.clone(), selected)
    };
    for _ in 0..3 {
        frame(vec![]);
    }
    let (_, rows, _) = frame(vec![]);
    assert!(!rows[1].enabled());
    let pos = rows[1].rect.center();
    for pressed in [true, false] {
        let (_, _, selected) = frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        assert_eq!(selected, Some(0));
    }
    ctx.memory_mut(|memory| memory.request_focus(rows[0].id));
    for (key, expected) in [
        (egui::Key::ArrowRight, 2),
        (egui::Key::Home, 0),
        (egui::Key::End, 2),
    ] {
        let (_, rows, selected) = frame(vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert_eq!(selected, Some(expected));
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(rows[expected].id)
        );
    }
}

#[test]
fn unavailable_selection_is_cleared_without_inventing_a_filter() {
    let ctx = egui::Context::default();
    let mut selected = Some(1);
    crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
        show(
            ui,
            "filters",
            &[
                Filter {
                    label: "All",
                    available: true,
                },
                Filter {
                    label: "Running",
                    available: false,
                },
            ],
            &mut selected,
            "Unavailable",
        );
    });
    assert_eq!(selected, None);
}

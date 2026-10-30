use super::*;

#[test]
fn source_disclosure_settles_without_sustained_discard() {
    for width in [760., 1440.] {
        let ctx = egui::Context::default();
        crate::presentation::layout::configure(&ctx);
        crate::presentation::player_theme::apply(&ctx);
        crate::presentation::typography::install(&ctx).unwrap();
        let mut state = state();
        state.compare_task = false;
        let condition = &mut state.package.as_mut().unwrap().package.items[0]
            .candidate
            .condition;
        condition.model_id = "bartowski/Llama-3.2-1B-Instruct-GGUF".into();
        condition.revision = "a".repeat(64);
        condition.engine_revision = "b".repeat(64);
        let item = &mut state.package.as_mut().unwrap().package.items[0];
        let mut baseline = item.candidate.clone();
        baseline.assets = vec![item.source.clone()];
        item.baseline = Some(baseline);
        item.source.asset_id = "source-disclosure-body-proof".into();
        state.assets.insert(
            ("a".repeat(64), "a".repeat(64)),
            Ok(Box::new(ReviewTextAsset {
                package_sha256: "a".repeat(64),
                asset_sha256: "a".repeat(64),
                bytes: crate::domain::api_values::DecimalU64(2000),
                media_type: "text/plain".into(),
                utf8: "한국어・日本語 long source document with wrapped lines\n".repeat(80),
            })),
        );
        let mut shell = crate::presentation::player_app::studio::StudioShell {
            selected: crate::presentation::player_app::studio::catalog::WORKSPACES
                .iter()
                .position(|workspace| workspace.id == "MODEL")
                .unwrap(),
            ..Default::default()
        };
        shell.reviews.state = state;
        let mut tick = 0;
        let mut frame = |events| {
            tick += 1;
            crate::presentation::test_support::run(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 900.),
                    )),
                    time: Some(tick as f64 / 60.),
                    events,
                    ..Default::default()
                },
                |ui| shell.show(ui, ""),
            )
        };
        let mut output = frame(vec![]);
        for _ in 0..5 {
            output = frame(vec![]);
        }
        let pos = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.job.text == "원본 자료" => {
                    Some(text.pos + text.galley.rect.center().to_vec2())
                }
                _ => None,
            })
            .expect("source disclosure must be visible");
        assert!(!output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::epaint::Shape::Text(text) if text.galley.job.text == "source-disclosure-body-proof"
        )), "source body must begin collapsed");
        frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        let mut consecutive = 0;
        let mut longest = 0;
        let mut counts = Vec::new();
        let mut saw_source_body = false;
        for _ in 0..30 {
            let output = frame(vec![]);
            saw_source_body |= output.shapes.iter().any(|shape| matches!(&shape.shape,
                egui::epaint::Shape::Text(text) if text.galley.job.text == "source-disclosure-body-proof"
            ));
            let discarded = !output.platform_output.request_discard_reasons.is_empty();
            consecutive = if discarded { consecutive + 1 } else { 0 };
            longest = longest.max(consecutive);
            counts.push(output.platform_output.num_completed_passes);
        }
        assert!(
            saw_source_body,
            "source disclosure click must open the body at {width}"
        );
        assert!(
            longest < 10,
            "sustained discard at {width}: {longest}; passes={counts:?}"
        );
        assert!(
            counts[25..].iter().all(|count| *count == 1),
            "idle layout at {width}: {counts:?}"
        );
    }
}

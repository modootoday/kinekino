use super::*;
use crate::application::control_visibility::ControlVisibility;
use eframe::egui::{pos2, vec2};
use std::time::Duration;

fn transport() -> Rect {
    Rect::from_min_size(pos2(0., 400.), vec2(960., 80.))
}
fn toolbar() -> Rect {
    Rect::from_min_size(pos2(0., 0.), vec2(960., 50.))
}

#[test]
fn stationary_transport_hover_keeps_controls_then_original_idle_hides_after_leaving() {
    let mut interaction = ChromeInteraction::default();
    let mut visibility = ControlVisibility::default();
    assert!(visibility.visible(Duration::ZERO, true, true, false));
    for second in [3, 10, 50] {
        interaction.remember(ControlRegion::Transport, transport());
        let state = interaction.observe(true, Some(pos2(40., 430.)));
        assert!(visibility.visible(
            Duration::from_secs(second),
            true,
            state.resumed,
            state.hovering_controls
        ));
    }
    interaction.remember(ControlRegion::Transport, transport());
    let video = interaction.observe(true, Some(pos2(400., 200.)));
    assert!(!video.hovering_controls);
    assert!(visibility.visible(
        Duration::from_secs(51),
        true,
        video.resumed,
        video.hovering_controls
    ));
    assert!(!visibility.visible(Duration::from_secs(53), true, false, false));
}

#[test]
fn old_regions_are_consumed_and_hidden_controls_cannot_pin_the_next_frame() {
    let mut interaction = ChromeInteraction::default();
    interaction.remember(ControlRegion::Toolbar, toolbar());
    assert!(
        interaction
            .observe(true, Some(pos2(10., 20.)))
            .hovering_controls
    );
    assert!(
        !interaction
            .observe(true, Some(pos2(10., 20.)))
            .hovering_controls
    );
    interaction.remember(ControlRegion::Transport, transport());
    assert!(!interaction.observe(true, None).hovering_controls);
}

#[test]
fn focus_return_reveals_controls_without_inventing_pointer_motion() {
    let mut interaction = ChromeInteraction::default();
    let mut visibility = ControlVisibility::default();
    interaction.observe(true, None);
    visibility.visible(Duration::ZERO, true, true, false);
    assert!(!visibility.visible(Duration::from_secs(4), true, false, false));
    assert!(!interaction.observe(false, None).resumed);
    let returned = interaction.observe(true, None);
    assert!(returned.resumed);
    assert!(!returned.hovering_controls);
    assert!(visibility.visible(
        Duration::from_secs(20),
        true,
        returned.resumed,
        returned.hovering_controls
    ));
    assert!(!interaction.observe(true, None).resumed);
    assert!(!visibility.visible(Duration::from_secs(23), true, false, false));
}

#[test]
fn unfocused_window_does_not_pin_a_stale_hover_or_repeat_focus_activity() {
    let mut interaction = ChromeInteraction::default();
    interaction.observe(true, None);
    interaction.remember(ControlRegion::Toolbar, toolbar());
    let inactive = interaction.observe(false, Some(pos2(20., 20.)));
    assert!(!inactive.hovering_controls && !inactive.resumed);
    let inactive_again = interaction.observe(false, Some(pos2(20., 20.)));
    assert!(!inactive_again.hovering_controls && !inactive_again.resumed);
}

//! The Developer page: the readouts it exposes and the tools it offers.

mod common;

use common::{Fixture, fixture_with_tasks};
use gpui_kit::TestAppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit_demo::storage::Settings;

fn open_developer(cx: &mut TestAppContext) -> Fixture {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-developer", cx));
    fixture
}

#[gpui_kit::test]
fn the_page_reports_the_running_build(cx: &mut TestAppContext) {
    let mut fixture = open_developer(cx);

    assert_eq!(
        fixture.label(cx, "page-title").as_deref(),
        Some("Developer tools")
    );
    assert!(fixture.has(cx, "dev-storage-path"));
    assert!(fixture.has(cx, "dev-state-json"));

    let json = fixture.label(cx, "dev-state-json").expect("state preview");
    assert!(json.contains("\"dark\""), "unexpected preview: {json}");
    assert!(json.contains("\"tasks\""));
}

#[gpui_kit::test]
fn the_state_preview_follows_the_workspace(cx: &mut TestAppContext) {
    let mut fixture = fixture_with_tasks(cx, &["Preview me"]);
    fixture.update(cx, |window, cx| window.click("nav-developer", cx));

    let json = fixture.label(cx, "dev-state-json").expect("state preview");
    assert!(
        json.contains("Preview me"),
        "the preview should include stored tasks: {json}"
    );
}

#[gpui_kit::test]
fn clearing_tasks_needs_a_second_click(cx: &mut TestAppContext) {
    let mut fixture = fixture_with_tasks(cx, &["Keep me", "Me too"]);
    fixture.update(cx, |window, cx| window.click("nav-developer", cx));
    fixture.scroll_to(cx, "dev-reset-tasks");

    fixture.update(cx, |window, cx| window.click("dev-reset-tasks", cx));
    assert_eq!(
        fixture.state(cx, |workspace| workspace.tasks.len()),
        2,
        "the first click should only arm the action"
    );
    assert_eq!(
        fixture.state(cx, |workspace| workspace.armed_reset),
        Some("tasks")
    );

    fixture.render(cx);
    fixture.update(cx, |window, cx| window.click("dev-reset-tasks", cx));
    assert!(fixture.state(cx, |workspace| workspace.tasks.is_empty()));
    assert_eq!(fixture.state(cx, |workspace| workspace.armed_reset), None);
    assert!(fixture.stored().tasks.is_empty());
}

#[gpui_kit::test]
fn resetting_preferences_restores_the_defaults(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(
        cx,
        Settings {
            dark: true,
            collapsed: true,
            animations: false,
            translucent: false,
            tasks: Vec::new(),
        },
    );
    fixture.update(cx, |window, cx| window.click("nav-developer", cx));
    fixture.scroll_to(cx, "dev-reset-preferences");

    fixture.update(cx, |window, cx| window.click("dev-reset-preferences", cx));
    fixture.render(cx);
    fixture.update(cx, |window, cx| window.click("dev-reset-preferences", cx));

    let settings = fixture.settings(cx);
    assert!(!settings.dark);
    assert!(!settings.collapsed);
    assert!(settings.animations);
    assert!(settings.translucent);
    assert_eq!(fixture.stored(), settings);
}

#[gpui_kit::test]
fn copying_the_state_confirms_itself(cx: &mut TestAppContext) {
    let mut fixture = open_developer(cx);
    fixture.scroll_to(cx, "dev-copy-state");

    assert!(!fixture.has(cx, "dev-copied"));
    fixture.update(cx, |window, cx| window.click("dev-copy-state", cx));

    assert!(fixture.state(cx, |workspace| workspace.copied_state));
    assert!(fixture.has(cx, "dev-copied"));
}

#[gpui_kit::test]
fn the_demo_triggers_move_real_state(cx: &mut TestAppContext) {
    let mut fixture = open_developer(cx);
    fixture.scroll_to(cx, "dev-toggle-theme");

    fixture.update(cx, |window, cx| window.click("dev-toggle-theme", cx));
    assert!(fixture.state(cx, |workspace| workspace.dark));

    fixture.update(cx, |window, cx| window.click("dev-toggle-rail", cx));
    assert!(fixture.state(cx, |workspace| workspace.collapsed));

    fixture.update(cx, |window, cx| window.click("dev-busy", cx));
    assert!(fixture.state(cx, |workspace| workspace.gallery_busy));
}

#[gpui_kit::test]
fn the_frame_counter_advances_with_rendering(cx: &mut TestAppContext) {
    let mut fixture = open_developer(cx);
    let before = fixture.state(cx, |workspace| workspace.renders);
    assert!(before > 0, "the page should report earlier frames");

    fixture.render(cx);
    fixture.render(cx);

    let after = fixture.state(cx, |workspace| workspace.renders);
    assert!(
        after > before,
        "rendering should advance the counter: {before} -> {after}"
    );
}

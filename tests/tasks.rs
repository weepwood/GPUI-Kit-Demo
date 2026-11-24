//! Tasks: adding, completing and removing entries through the real controls,
//! plus the persistence that makes them survive a restart.

mod common;

use common::{Fixture, fixture_with_tasks};
use gpui_kit::TestAppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit_demo::Page;
use gpui_kit_demo::storage::Settings;

#[gpui_kit::test]
fn the_editor_adds_a_task(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));

    assert!(
        fixture.has(cx, "task-empty"),
        "the empty state should show first"
    );

    fixture.update(cx, |window, cx| {
        window.click("task-input", cx);
        window.input("Ship the design system", cx);
    });
    assert_eq!(
        fixture.value(cx, "task-input").as_deref(),
        Some("Ship the design system")
    );

    fixture.update(cx, |window, cx| window.click("add-task", cx));

    assert_eq!(fixture.state(cx, |workspace| workspace.tasks.len()), 1);
    assert_eq!(
        fixture.label(cx, "task-summary").as_deref(),
        Some("1 saved, 0 done")
    );
    assert!(fixture.has(cx, "task-row-0"));
    assert!(
        !fixture.has(cx, "task-empty"),
        "the empty state should be gone"
    );
    assert_eq!(
        fixture.value(cx, "task-input").as_deref(),
        Some(""),
        "the field should be cleared after adding"
    );
}

#[gpui_kit::test]
fn adding_a_task_writes_it_to_the_settings_file(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));
    fixture.update(cx, |window, cx| {
        window.click("task-input", cx);
        window.input("Persist me", cx);
        window.click("add-task", cx);
    });

    let stored = fixture.stored();
    assert_eq!(stored.tasks.len(), 1);
    assert_eq!(stored.tasks[0].title, "Persist me");
    assert!(!stored.tasks[0].done);
}

#[gpui_kit::test]
fn the_editor_ignores_blank_input(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));
    fixture.update(cx, |window, cx| {
        window.click("task-input", cx);
        window.input("   ", cx);
        window.click("add-task", cx);
    });

    assert!(fixture.state(cx, |workspace| workspace.tasks.is_empty()));
    assert!(fixture.has(cx, "task-empty"));
}

#[gpui_kit::test]
fn checking_a_task_marks_it_done_everywhere(cx: &mut TestAppContext) {
    let mut fixture = fixture_with_tasks(cx, &["Write the release notes"]);
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));

    assert_eq!(fixture.checked(cx, "task-done-0"), Some(false));
    fixture.update(cx, |window, cx| window.click("task-done-0", cx));

    assert_eq!(fixture.checked(cx, "task-done-0"), Some(true));
    assert_eq!(
        fixture.label(cx, "task-summary").as_deref(),
        Some("1 saved, 1 done")
    );
    assert!(
        fixture.stored().tasks[0].done,
        "the done flag should be persisted"
    );

    // The Home page reports the same numbers.
    fixture.update(cx, |window, cx| window.click("nav-overview", cx));
    assert_eq!(fixture.try_label(cx, "task-summary"), None);
    assert_eq!(
        fixture.label(cx, "overview-task-count").as_deref(),
        Some("1"),
        "the counter should already show the settled total"
    );
}

#[gpui_kit::test]
fn removing_a_task_restores_the_empty_state(cx: &mut TestAppContext) {
    let mut fixture = fixture_with_tasks(cx, &["One task"]);
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));
    assert!(fixture.has(cx, "task-row-0"));

    fixture.update(cx, |window, cx| window.click("remove-task-0", cx));

    assert!(fixture.state(cx, |workspace| workspace.tasks.is_empty()));
    assert!(!fixture.has(cx, "task-row-0"));
    assert!(fixture.has(cx, "task-empty"));
    assert!(fixture.stored().tasks.is_empty());
}

#[gpui_kit::test]
fn the_shortcut_jumps_to_the_editor_and_focuses_it(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.focus_shell(cx);

    fixture.update(cx, |window, cx| window.press("ctrl-n", cx));
    fixture.render(cx);

    assert_eq!(
        fixture.state(cx, |workspace| workspace.page),
        Page::Activity
    );
    assert_eq!(
        fixture.focused(cx, "task-input"),
        Some(true),
        "Ctrl+N should put the caret in the new-task field"
    );

    fixture.update(cx, |window, cx| window.input("Typed straight in", cx));
    assert_eq!(
        fixture.value(cx, "task-input").as_deref(),
        Some("Typed straight in")
    );
}

#[gpui_kit::test]
fn stored_tasks_are_restored_into_the_list(cx: &mut TestAppContext) {
    let mut fixture = fixture_with_tasks(cx, &["First", "Second"]);
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));

    assert_eq!(
        fixture.label(cx, "task-summary").as_deref(),
        Some("2 saved, 0 done")
    );
    // Row ids carry the storage index; the list itself renders newest first.
    assert_eq!(
        fixture.label(cx, "task-row-0").as_deref(),
        Some("Task: First")
    );
    assert_eq!(
        fixture.label(cx, "task-row-1").as_deref(),
        Some("Task: Second")
    );
    let (first_y, second_y) = (
        fixture.rect(cx, "task-row-0").1,
        fixture.rect(cx, "task-row-1").1,
    );
    assert!(
        second_y < first_y,
        "the newest task should render above the older one"
    );
}

#[gpui_kit::test]
fn compact_density_shortens_the_rows(cx: &mut TestAppContext) {
    let mut fixture = fixture_with_tasks(cx, &["A task with some body"]);
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));
    let comfortable = fixture.rect(cx, "task-row-0").3;

    fixture.update(cx, |window, cx| window.click("nav-components", cx));
    // `RadioGroup` re-keys its children by index, so the second radio is
    // addressed by position inside the group rather than by name.
    fixture.scroll_to(cx, "density-control");
    fixture.update(cx, |window, cx| {
        window.within("density-radio").click(1usize, cx)
    });
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));
    let compact = fixture.rect(cx, "task-row-0").3;

    assert!(
        compact < comfortable,
        "compact rows ({compact}) should be shorter than comfortable rows ({comfortable})"
    );
    assert_eq!(
        fixture.label(cx, "task-density").as_deref(),
        Some("Compact density")
    );
}

//! Layout invariants.
//!
//! There is no pixel-snapshot support in GPUI, so these tests guard the
//! geometry that a redesign must not break: the shell fills the window exactly,
//! the status bar stays pinned, and nothing escapes the content column.

mod common;

use common::{Fixture, fixture_with_tasks};
use gpui_kit::TestAppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit_demo::storage::Settings;

const WINDOW_WIDTH: f32 = 1220.;
const WINDOW_HEIGHT: f32 = 800.;

#[gpui_kit::test]
fn the_rail_and_the_content_share_the_window(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open_calm(cx);

    let (rail_x, _, rail_width, _) = fixture.rect(cx, "sidebar");
    let (content_x, _, content_width, _) = fixture.rect(cx, "workspace-scroll");

    assert_eq!(rail_x, 0., "the rail should start at the window edge");
    assert_eq!(
        rail_x + rail_width,
        content_x,
        "the content area should begin where the rail ends"
    );
    assert_eq!(
        content_x + content_width,
        WINDOW_WIDTH,
        "the content area should reach the right edge"
    );
}

#[gpui_kit::test]
fn the_status_bar_stays_pinned_to_the_bottom(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open_calm(cx);

    let (_, top, _, height) = fixture.rect(cx, "status-bar");
    assert_eq!(
        top + height,
        WINDOW_HEIGHT,
        "the status bar should sit on the window floor"
    );

    let (_, content_top, _, content_height) = fixture.rect(cx, "workspace-scroll");
    assert!(
        content_top + content_height <= top,
        "the content area should end above the status bar"
    );
}

#[gpui_kit::test]
fn navigation_targets_clear_the_minimum_size(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open_calm(cx);

    for id in [
        "nav-overview",
        "nav-components",
        "nav-activity",
        "nav-settings",
        "sidebar-toggle",
        "theme-button",
    ] {
        let (_, _, width, height) = fixture.rect(cx, id);
        assert!(
            height >= 24. && width >= 24.,
            "{id} is too small to hit comfortably: {width}x{height}"
        );
    }
}

#[gpui_kit::test]
fn nothing_escapes_the_content_column(cx: &mut TestAppContext) {
    let mut fixture = fixture_with_tasks(cx, &["One", "Two"]);
    let (content_x, _, content_width, _) = fixture.rect(cx, "workspace-scroll");
    let right_edge = content_x + content_width;

    for page in [
        "nav-overview",
        "nav-components",
        "nav-activity",
        "nav-settings",
    ] {
        fixture.update(cx, |window, cx| window.click(page, cx));
        fixture.render(cx);

        for id in ["page-title", "page-eyebrow", "status-bar"] {
            if !fixture.has(cx, id) {
                continue;
            }
            let (x, _, width, _) = fixture.rect(cx, id);
            assert!(
                x >= content_x - 0.5 && x + width <= right_edge + 0.5,
                "{id} escapes the content column on {page}: x={x} width={width}"
            );
        }
    }
}

#[gpui_kit::test]
fn the_window_minimum_size_is_respected_by_the_shell(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    let (_, _, width, _) = fixture.rect(cx, "sidebar");
    assert!(
        width > 0. && width <= WINDOW_WIDTH,
        "the rail should occupy part of the window, got {width}"
    );
    assert!(fixture.has(cx, "workspace-root"));
}

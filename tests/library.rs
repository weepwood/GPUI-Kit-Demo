//! Library: the live demos, which are the page most likely to drift because
//! every control there is bound to real state.

mod common;

use common::Fixture;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{Point, TestAppContext, px};
use gpui_kit_demo::storage::Settings;

#[gpui_kit::test]
fn the_character_count_follows_the_controlled_input(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-components", cx));

    fixture.scroll_to(cx, "library-input");
    assert_eq!(
        fixture.label(cx, "library-characters").as_deref(),
        Some("0 characters")
    );

    fixture.click_and_focus(cx, "library-input");
    fixture.update(cx, |window, cx| window.input("hello", cx));

    assert_eq!(fixture.value(cx, "library-input").as_deref(), Some("hello"));
    assert_eq!(
        fixture.label(cx, "library-characters").as_deref(),
        Some("5 characters")
    );
}

#[gpui_kit::test]
fn the_count_checkbox_hides_and_restores_the_pill(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-components", cx));
    fixture.scroll_to(cx, "gallery-count-checkbox");

    assert_eq!(fixture.checked(cx, "gallery-count-checkbox"), Some(true));
    fixture.update(cx, |window, cx| window.click("gallery-count-checkbox", cx));

    assert!(!fixture.state(cx, |workspace| workspace.show_counts));
    assert_eq!(
        fixture.try_label(cx, "library-characters"),
        None,
        "the pill should be removed, not just hidden"
    );

    fixture.update(cx, |window, cx| window.click("gallery-count-checkbox", cx));
    assert!(fixture.state(cx, |workspace| workspace.show_counts));
    assert!(fixture.has(cx, "library-characters"));
}

#[gpui_kit::test]
fn the_slider_drives_the_progress_readout(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-components", cx));
    fixture.scroll_to(cx, "library-slider");

    assert_eq!(
        fixture.label(cx, "library-slider-value").as_deref(),
        Some("40 percent")
    );

    let (_, _, width, height) = fixture.rect(cx, "library-slider");
    fixture.update(cx, |window, cx| {
        window.click_at(
            "library-slider",
            Point {
                x: px(width * 0.8),
                y: px(height / 2.),
            },
            cx,
        )
    });

    let value = fixture.state(cx, |workspace| workspace.progress_value);
    assert!(
        value > 50.,
        "clicking near the end of the track should raise the value, got {value}"
    );
    assert_eq!(
        fixture.label(cx, "library-slider-value").as_deref(),
        Some(format!("{} percent", value.round() as i32).as_str())
    );
    assert!(fixture.has(cx, "gallery-progress"));
}

#[gpui_kit::test]
fn the_gallery_switches_mirror_the_workspace(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-components", cx));
    fixture.scroll_to(cx, "gallery-collapse-switch");

    fixture.update(cx, |window, cx| window.click("gallery-collapse-switch", cx));
    assert!(fixture.state(cx, |workspace| workspace.collapsed));
    assert_eq!(fixture.selected(cx, "nav-overview"), Some(false));

    fixture.update(cx, |window, cx| window.click("gallery-dark-switch", cx));
    assert!(fixture.state(cx, |workspace| workspace.dark));
    assert!(fixture.stored().dark);
}

#[gpui_kit::test]
fn every_gallery_button_is_clickable(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-components", cx));

    let mut clicked = 0;
    for id in ["btn-primary", "btn-secondary", "btn-ghost", "btn-link"] {
        if !fixture.has(cx, id) {
            continue;
        }
        fixture.update(cx, |window, cx| window.click(id, cx));
        clicked += 1;
    }

    assert_eq!(clicked, 4);
    assert_eq!(
        fixture.state(cx, |workspace| workspace.clicks),
        2,
        "only the primary and secondary buttons record a click"
    );
}

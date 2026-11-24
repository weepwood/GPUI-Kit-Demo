//! Preferences: the switches, the storage path they write to, and the build
//! identity the page reports.

mod common;

use common::Fixture;
use gpui_kit::TestAppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit_demo::storage::Settings;

#[gpui_kit::test]
fn the_theme_switch_flips_the_palette_and_persists(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    assert!(!fixture.state(cx, |workspace| workspace.dark));

    fixture.update(cx, |window, cx| window.click("theme-button", cx));
    assert!(fixture.state(cx, |workspace| workspace.dark));
    assert!(fixture.stored().dark, "the theme should be written to disk");

    fixture.update(cx, |window, cx| window.click("theme-button", cx));
    assert!(!fixture.state(cx, |workspace| workspace.dark));
}

#[gpui_kit::test]
fn the_dark_setting_switch_matches_the_header_button(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-settings", cx));

    fixture.update(cx, |window, cx| window.click("dark-switch", cx));
    assert!(fixture.state(cx, |workspace| workspace.dark));
    assert_eq!(fixture.checked(cx, "dark-switch"), Some(true));

    // The header button is the same state, reached from the other end.
    fixture.update(cx, |window, cx| window.click("theme-button", cx));
    assert!(!fixture.state(cx, |workspace| workspace.dark));
    assert_eq!(fixture.checked(cx, "dark-switch"), Some(false));
}

#[gpui_kit::test]
fn motion_can_be_switched_off(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-settings", cx));

    fixture.update(cx, |window, cx| window.click("animations-switch", cx));
    assert!(!fixture.state(cx, |workspace| workspace.animations));
    assert!(!fixture.stored().animations);

    // With motion off no page is wrapped in the entrance animation.
    fixture.update(cx, |window, cx| window.click("nav-components", cx));
    assert_eq!(
        fixture.try_label(cx, "page-enter-1"),
        None,
        "static pages should not mount the entrance wrapper"
    );
    assert_eq!(
        fixture.label(cx, "page-title").as_deref(),
        Some("Design kit")
    );
}

#[gpui_kit::test]
fn the_frosted_switch_is_remembered(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-settings", cx));
    assert!(fixture.state(cx, |workspace| workspace.translucent));

    fixture.scroll_to(cx, "translucent-switch");
    fixture.update(cx, |window, cx| window.click("translucent-switch", cx));

    assert!(!fixture.state(cx, |workspace| workspace.translucent));
    assert!(!fixture.stored().translucent);
}

#[gpui_kit::test]
fn the_collapsed_setting_switch_moves_the_rail(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open_calm(cx);
    let expanded = fixture.rect(cx, "sidebar").2;
    fixture.update(cx, |window, cx| window.click("nav-settings", cx));

    fixture.update(cx, |window, cx| window.click("sidebar-switch", cx));

    assert!(fixture.state(cx, |workspace| workspace.collapsed));
    assert!(fixture.rect(cx, "sidebar").2 < expanded);
    assert!(fixture.stored().collapsed);
}

#[gpui_kit::test]
fn the_page_shows_where_settings_live(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-settings", cx));

    let shown = fixture
        .label(cx, "storage-path")
        .expect("the storage path should be exposed");
    assert!(
        shown.ends_with("settings.json"),
        "unexpected storage path: {shown}"
    );

    let version = fixture.label(cx, "app-version").expect("version line");
    assert!(
        version.contains(env!("CARGO_PKG_VERSION")),
        "the version line should match the crate version: {version}"
    );
}

#[gpui_kit::test]
fn every_setting_row_is_observable(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-settings", cx));

    for (id, label) in [
        ("row-appearance", "Dark appearance"),
        ("row-navigation", "Collapsed sidebar"),
        ("row-motion", "Interface motion"),
        ("row-frosted", "Frosted background"),
    ] {
        fixture.require(cx, id);
        assert_eq!(fixture.label(cx, id).as_deref(), Some(label));
    }
}

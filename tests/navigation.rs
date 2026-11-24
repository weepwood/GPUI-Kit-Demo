//! Navigation: clicking the rail, the keyboard bindings, and what each page
//! reports about itself.

mod common;

use common::Fixture;
use gpui_kit::TestAppContext;
use gpui_kit::test::TestWindowExt;
use gpui_kit_demo::Page;
use gpui_kit_demo::storage::Settings;

#[gpui_kit::test]
fn every_destination_switches_the_page(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());

    for (id, page, title) in [
        ("nav-overview", Page::Overview, "Home"),
        ("nav-components", Page::Components, "Library"),
        ("nav-activity", Page::Activity, "Tasks"),
        ("nav-settings", Page::Settings, "Preferences"),
        ("nav-developer", Page::Developer, "Developer"),
    ] {
        fixture.update(cx, |window, cx| window.click(id, cx));

        assert_eq!(fixture.state(cx, |workspace| workspace.page), page);
        assert_eq!(
            fixture.label(cx, "page-title").as_deref(),
            Some(match page {
                Page::Overview => "Workspace overview",
                Page::Components => "Design kit",
                Page::Activity => "Tasks",
                Page::Settings => "Appearance & layout",
                Page::Developer => "Developer tools",
            }),
            "the hero of {title} should name the page"
        );
        assert!(
            fixture
                .label(cx, "page-eyebrow")
                .is_some_and(|label| label.contains('/') || label == "COMPONENT LIBRARY"),
            "the eyebrow of {title} should be set"
        );
    }
}

#[gpui_kit::test]
fn the_active_destination_is_marked_selected(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.update(cx, |window, cx| window.click("nav-activity", cx));

    let selected: Vec<(String, bool)> = fixture.update(cx, |window, _| {
        [
            "nav-overview",
            "nav-components",
            "nav-activity",
            "nav-settings",
            "nav-developer",
        ]
        .iter()
        .map(|id| {
            (
                (*id).to_string(),
                window.find(*id).selected().unwrap_or(false),
            )
        })
        .collect()
    });

    assert_eq!(
        selected,
        vec![
            ("nav-overview".to_string(), false),
            ("nav-components".to_string(), false),
            ("nav-activity".to_string(), true),
            ("nav-settings".to_string(), false),
            ("nav-developer".to_string(), false),
        ]
    );
}

#[gpui_kit::test]
fn keyboard_shortcuts_reach_the_same_pages(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());
    fixture.focus_shell(cx);

    for (key, page) in [
        ("ctrl-2", Page::Components),
        ("ctrl-3", Page::Activity),
        ("ctrl-4", Page::Settings),
        ("ctrl-5", Page::Developer),
        ("ctrl-1", Page::Overview),
    ] {
        fixture.update(cx, |window, cx| window.press(key, cx));
        assert_eq!(
            fixture.state(cx, |workspace| workspace.page),
            page,
            "{key} should open {page:?}"
        );
    }
}

#[gpui_kit::test]
fn the_sidebar_collapses_from_the_rail_and_the_shortcut(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open_calm(cx);
    let expanded = fixture.rect(cx, "sidebar").2;

    fixture.update(cx, |window, cx| window.click("sidebar-toggle", cx));
    assert!(fixture.state(cx, |workspace| workspace.collapsed));
    let collapsed = fixture.rect(cx, "sidebar").2;
    assert!(
        collapsed < expanded,
        "collapsed rail {collapsed} should be narrower than {expanded}"
    );

    fixture.focus_shell(cx);
    fixture.update(cx, |window, cx| window.press("ctrl-b", cx));
    fixture.render(cx);
    assert!(!fixture.state(cx, |workspace| workspace.collapsed));
    assert_eq!(fixture.rect(cx, "sidebar").2, expanded);
}

#[gpui_kit::test]
fn the_collapsed_rail_keeps_its_icons_centred(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open_calm(cx);
    fixture.update(cx, |window, cx| window.click("sidebar-toggle", cx));

    let (rail_x, _, rail_width, _) = fixture.rect(cx, "sidebar");
    let (icon_x, _, icon_width, _) = fixture.rect(cx, "nav-overview");
    let rail_centre = rail_x + rail_width / 2.;
    let icon_centre = icon_x + icon_width / 2.;

    assert!(
        (rail_centre - icon_centre).abs() <= 1.,
        "the icon column should sit on the rail centre line: rail {rail_centre} vs icon {icon_centre}"
    );
}

#[gpui_kit::test]
fn the_home_destination_cards_open_their_pages(cx: &mut TestAppContext) {
    let mut fixture = Fixture::open(cx, Settings::default());

    fixture.update(cx, |window, cx| window.click("overview-components", cx));
    assert_eq!(
        fixture.state(cx, |workspace| workspace.page),
        Page::Components
    );

    fixture.focus_shell(cx);
    fixture.update(cx, |window, cx| window.press("ctrl-1", cx));
    fixture.render(cx);
    fixture.update(cx, |window, cx| window.click("overview-preferences", cx));
    assert_eq!(
        fixture.state(cx, |workspace| workspace.page),
        Page::Settings
    );
}

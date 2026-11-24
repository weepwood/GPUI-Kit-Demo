//! Shared fixture for the headless UI tests.
//!
//! Every test opens the real workbench in a `TestAppContext` at a fixed window
//! size, with settings stored in a throwaway directory, so a run never touches
//! the developer's own configuration and layout assertions stay deterministic.
#![allow(dead_code)]

use gpui_kit::base::Root;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{
    App, AppContext as _, Bounds, Entity, Pixels, Point, Render, ScrollDelta, Size, TestAppContext,
    Window, WindowBounds, WindowHandle, WindowOptions, px, size,
};

use gpui_kit_demo::storage::{Settings, TaskEntry};
use gpui_kit_demo::{Workspace, window_options};

/// The window size every test mounts at: wide enough for the two-column Home
/// layout, small enough to keep a frame cheap.
pub const WINDOW_WIDTH: f32 = 1220.;
pub const WINDOW_HEIGHT: f32 = 800.;

pub fn window_size() -> Size<Pixels> {
    size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT))
}

/// A workbench mounted in a test window, plus its throwaway settings file.
pub struct Fixture {
    pub handle: WindowHandle<Root>,
    pub workspace: Entity<Workspace>,
    pub settings_path: std::path::PathBuf,
}

impl Fixture {
    /// Open the workbench with `settings` in a fixed-size test window.
    pub fn open(cx: &mut TestAppContext, settings: Settings) -> Self {
        let settings_path = temporary_settings_path();
        cx.update(|cx| gpui_kit_demo::init(cx, &settings));

        let (handle, workspace) = cx.update(|cx| {
            let bounds = Bounds {
                origin: Point::default(),
                size: window_size(),
            };
            // Reuse the real window options so the test shell matches the app.
            let options = window_options(&settings, WindowBounds::Windowed(bounds));
            let store = settings_path.clone();
            gpui_kit::open_window(options, cx, move |window, cx| {
                cx.new(|cx| Workspace::new(window, cx, settings, None).with_store(store))
            })
            .expect("open the workbench window")
        });

        let mut fixture = Self {
            handle: handle.downcast::<Root>().expect("base Root"),
            workspace,
            settings_path,
        };
        fixture.render(cx);
        fixture
    }

    /// Open the workbench with motion disabled, so one frame already carries
    /// the settled layout. Use this for geometry assertions.
    pub fn open_calm(cx: &mut TestAppContext) -> Self {
        Self::open(
            cx,
            Settings {
                animations: false,
                ..Default::default()
            },
        )
    }

    /// Focus the workbench's focus home, which is where the keybindings live.
    pub fn focus_shell(&mut self, cx: &mut TestAppContext) {
        let handle = self.state(cx, |workspace| workspace.focus_handle());
        self.update(cx, |window, cx| window.focus(&handle, cx));
        self.render(cx);
    }

    /// Open any view in a bare test window; used by layout helpers.
    pub fn open_view<V: Render>(
        cx: &mut TestAppContext,
        build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    ) -> (WindowHandle<Root>, Entity<V>) {
        cx.update(|cx| {
            let bounds = Bounds {
                origin: Point::default(),
                size: window_size(),
            };
            let (window, content) = gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                cx,
                build,
            )
            .expect("open a test window");
            (window.downcast::<Root>().expect("base Root"), content)
        })
    }

    /// Build a frame and hand the window to `f`.
    pub fn update<R>(
        &mut self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut Window, &mut App) -> R,
    ) -> R {
        cx.update_window(self.handle.into(), |_, window, cx| f(window, cx))
            .expect("update the test window")
    }

    /// Refresh the recorded element facts without running an assertion body.
    pub fn render(&mut self, cx: &mut TestAppContext) {
        self.update(cx, |window, cx| window.render_frame(cx));
    }

    /// The settings the workspace would persist right now.
    pub fn settings(&self, cx: &mut TestAppContext) -> Settings {
        let workspace = self.workspace.clone();
        cx.update(|cx| workspace.read(cx).current_settings())
    }

    /// The settings file as parsed from disk.
    pub fn stored(&self) -> Settings {
        gpui_kit_demo::storage::load_from(&self.settings_path).expect("read the settings file")
    }

    /// Read the live workspace state.
    pub fn state<R>(&self, cx: &mut TestAppContext, f: impl FnOnce(&Workspace) -> R) -> R {
        let workspace = self.workspace.clone();
        cx.update(|cx| f(workspace.read(cx)))
    }

    /// `(x, y, width, height)` of an element, in pixels.
    pub fn rect(&mut self, cx: &mut TestAppContext, id: &'static str) -> (f32, f32, f32, f32) {
        self.update(cx, |window, _| {
            let bounds = window.find(id).bounds();
            (
                f32::from(bounds.origin.x),
                f32::from(bounds.origin.y),
                f32::from(bounds.size.width),
                f32::from(bounds.size.height),
            )
        })
    }

    /// The accessibility label currently exposed by `id`.
    pub fn label(&mut self, cx: &mut TestAppContext, id: &'static str) -> Option<String> {
        self.update(cx, |window, _| window.find(id).label().map(str::to_string))
    }

    /// Whether `id` is registered in the last frame.
    pub fn has(&mut self, cx: &mut TestAppContext, id: &'static str) -> bool {
        self.update(cx, |window, _| window.try_find(id).is_some())
    }

    /// The native checked state of a control, if it exposes one.
    pub fn checked(&mut self, cx: &mut TestAppContext, id: &'static str) -> Option<bool> {
        self.update(cx, |window, _| window.find(id).checked())
    }

    /// The native selected state of a control, if it exposes one.
    pub fn selected(&mut self, cx: &mut TestAppContext, id: &'static str) -> Option<bool> {
        self.update(cx, |window, _| window.find(id).selected())
    }

    /// The native value of a control, if it exposes one.
    pub fn value(&mut self, cx: &mut TestAppContext, id: &'static str) -> Option<String> {
        self.update(cx, |window, _| window.find(id).value().map(str::to_string))
    }

    /// Whether a control currently holds keyboard focus.
    pub fn focused(&mut self, cx: &mut TestAppContext, id: &'static str) -> Option<bool> {
        self.update(cx, |window, _| window.find(id).focused())
    }

    /// The accessibility label of `id`, or `None` when it is not registered.
    pub fn try_label(&mut self, cx: &mut TestAppContext, id: &'static str) -> Option<String> {
        self.update(cx, |window, _| {
            window
                .try_find(id)
                .and_then(|element| element.label().map(str::to_string))
        })
    }

    /// Scroll the page until `id` sits fully inside the content viewport.
    ///
    /// Long pages (Library especially) start with their lower cards clipped, and
    /// the harness refuses to click a target that is not fully visible, so
    /// "partly on screen" is not good enough.
    pub fn scroll_to(&mut self, cx: &mut TestAppContext, id: &'static str) {
        for _ in 0..80 {
            let (_, view_top, _, view_height) = self.rect(cx, "workspace-scroll");
            let (_, top, _, height) = self.rect(cx, id);
            let inside = top >= view_top && top + height <= view_top + view_height;
            if inside {
                return;
            }
            // A positive delta scrolls the content back up, a negative one down.
            let lines = if top < view_top { 4. } else { -4. };
            self.update(cx, |window, cx| {
                window.scroll(
                    "workspace-scroll",
                    ScrollDelta::Lines(Point { x: 0., y: lines }),
                    cx,
                )
            });
            self.render(cx);
        }
        panic!("could not scroll `{id}` into view");
    }

    /// Click `id` until it reports focus.
    ///
    /// Scrolled content settles over more than one frame, so a single click can
    /// land on the position the target had a moment earlier; retrying after a
    /// fresh frame is deterministic and mirrors what a user does.
    pub fn click_and_focus(&mut self, cx: &mut TestAppContext, id: &'static str) {
        for attempt in 0..4 {
            self.update(cx, |window, cx| window.click(id, cx));
            self.render(cx);
            if self.focused(cx, id) == Some(true) {
                return;
            }
            assert!(
                attempt < 3,
                "`{id}` never took keyboard focus; is it clickable in the current frame?"
            );
        }
    }

    /// Assert that `id` is registered, with an actionable message when the
    /// `test-support` feature is missing.
    pub fn require(&mut self, cx: &mut TestAppContext, id: &'static str) {
        let present = self.update(cx, |window, _| window.try_find(id).is_some());
        assert!(
            present,
            "element `{id}` is not registered: run the suite with \
             `cargo test --features gpui-kit/test-support`"
        );
    }
}

/// Open a workbench pre-filled with `titles` (all undone).
pub fn fixture_with_tasks(cx: &mut TestAppContext, titles: &[&str]) -> Fixture {
    let settings = Settings {
        tasks: titles
            .iter()
            .map(|title| TaskEntry::new((*title).into()))
            .collect(),
        ..Default::default()
    };
    Fixture::open(cx, settings)
}

/// A unique, empty settings path under the system temp directory.
pub fn temporary_settings_path() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let serial = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut dir = std::env::temp_dir();
    dir.push("gpui-kit-demo-tests");
    std::fs::create_dir_all(&dir).expect("create the test settings directory");
    dir.push(format!("settings-{}-{serial}.json", std::process::id()));
    let _ = std::fs::remove_file(&dir);
    dir
}

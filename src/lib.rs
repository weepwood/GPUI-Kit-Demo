//! GPUI Kit Demo — a reusable native desktop workbench built on GPUI Kit.
//!
//! The crate is a library so the workbench can be mounted by the binary
//! (`src/main.rs`) and by the headless UI tests in `tests/`. Everything a
//! test needs is public: [`init`], [`open_workbench`], [`window_options`],
//! [`Workspace`], [`Page`] and the [`storage`] model. Everything else stays
//! private.

mod gallery;
mod kit;
mod palette;
pub mod storage;

use gpui_kit::assets::Assets;
use gpui_kit::base::StyledExt as _;
// `.test_support()` registers an element for UI tests. It is a zero-cost
// passthrough outside `cargo test`, so it can live in the production tree.
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::slider::{SliderEvent, SliderState};
use gpui_kit::component::{
    Icon, IconName, Sizable, Theme, ThemeMode,
    button::*,
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputState},
    switch::Switch,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kit::{caption, card, divider, pill, setting_row};
use palette::Palette;
use std::path::PathBuf;
use std::time::Duration;
use storage::MAX_TASKS;

// Window navigation actions, bound globally at startup. The Library page
// documents these same bindings.
gpui_kit::actions!(
    workspace,
    [
        NavigateHome,
        NavigateLibrary,
        NavigateTasks,
        NavigatePreferences,
        NavigateDeveloper,
        FocusNewTask,
        ToggleSidebar,
    ]
);

/// One navigation card on the Home page.
#[derive(Clone)]
struct Destination {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    icon: IconName,
    page: Page,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Overview,
    Components,
    Activity,
    Settings,
    Developer,
}

impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Home",
            Self::Components => "Library",
            Self::Activity => "Tasks",
            Self::Settings => "Preferences",
            Self::Developer => "Developer",
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Overview => IconName::LayoutDashboard,
            Self::Components => IconName::GalleryVerticalEnd,
            Self::Activity => IconName::Inbox,
            Self::Settings => IconName::Settings,
            Self::Developer => IconName::Cpu,
        }
    }

    fn index(self) -> u8 {
        match self {
            Self::Overview => 0,
            Self::Components => 1,
            Self::Activity => 2,
            Self::Settings => 3,
            Self::Developer => 4,
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::Overview => "WORKBENCH / OVERVIEW",
            Self::Components => "WORKBENCH / COMPONENTS",
            Self::Activity => "WORKBENCH / TASKS",
            Self::Settings => "WORKBENCH / PREFERENCES",
            Self::Developer => "WORKBENCH / DEVELOPER",
        }
    }

    fn lead(self) -> &'static str {
        match self {
            Self::Overview => "Local tasks, native controls, and appearance settings.",
            Self::Components => "Native interaction patterns and their actual states.",
            Self::Activity => "A small, persistent task list for trying native interactions.",
            Self::Settings => "A small set of durable, device-specific preferences.",
            Self::Developer => "Runtime facts, stored state and the debugging tools.",
        }
    }
}

/// The workbench view. Fields are public on purpose: this crate doubles as a
/// reference project, and the headless UI tests in `tests/` read the same
/// state the screens render.
pub struct Workspace {
    pub page: Page,
    pub dark: bool,
    pub collapsed: bool,
    pub animations: bool,
    pub translucent: bool,
    pub clicks: usize,
    pub tasks: Vec<storage::TaskEntry>,
    input: Entity<InputState>,
    playground_input: Entity<InputState>,
    pub save_error: Option<String>,
    /// Library demos — real, in-memory state (not persisted by design).
    pub gallery_busy: bool,
    pub saves_done: usize,
    pub show_counts: bool,
    pub density: usize,
    progress: Entity<SliderState>,
    pub progress_value: f32,
    /// Focus home of the workbench root; action dispatch rides the focus path.
    focus: FocusHandle,
    /// Id of the destination row currently hovered, for the arrow nudge.
    pub hovered_destination: Option<&'static str>,
    /// Frames rendered since launch, shown on the Developer page.
    pub renders: usize,
    /// Launch time, for the uptime readout.
    started: std::time::Instant,
    /// Which destructive Developer action is waiting for its second click.
    pub armed_reset: Option<&'static str>,
    /// Whether the settings JSON was just copied to the clipboard.
    pub copied_state: bool,
    /// Overrides the settings file location. Tests point it at a temporary
    /// directory so a run never rewrites the real user profile.
    store: Option<PathBuf>,
}

impl Workspace {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        settings: storage::Settings,
        save_error: Option<String>,
    ) -> Self {
        let input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Add a task to your workspace"));
        let playground_input = cx
            .new(|cx| InputState::new(window, cx).placeholder("Type to preview the native input"));

        let progress = cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(100.)
                .step(1.)
                .default_value(40.)
        });
        cx.subscribe(&progress, |this, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change(value) = event {
                this.progress_value = value.end();
                cx.notify();
            }
        })
        .detach();

        Self {
            page: Page::Overview,
            dark: settings.dark,
            collapsed: settings.collapsed,
            animations: settings.animations,
            translucent: settings.translucent,
            clicks: 0,
            tasks: settings.tasks,
            input,
            playground_input,
            save_error,
            gallery_busy: false,
            saves_done: 0,
            show_counts: true,
            density: 0,
            progress,
            progress_value: 40.,
            focus: cx.focus_handle(),
            hovered_destination: None,
            renders: 0,
            started: std::time::Instant::now(),
            armed_reset: None,
            copied_state: false,
            store: None,
        }
    }

    /// Store settings at `path` instead of the platform config directory.
    pub fn with_store(mut self, path: impl Into<PathBuf>) -> Self {
        self.store = Some(path.into());
        self
    }

    /// The workbench's focus home: the element that owns the keybindings.
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }

    /// The settings this workspace would persist right now.
    pub fn current_settings(&self) -> storage::Settings {
        storage::Settings {
            dark: self.dark,
            collapsed: self.collapsed,
            animations: self.animations,
            translucent: self.translucent,
            tasks: self.tasks.clone(),
        }
    }

    fn persist(&mut self) {
        let settings = self.current_settings();
        let result = match &self.store {
            Some(path) => storage::save_to(path, &settings),
            None => storage::save(&settings),
        };
        self.save_error = result.err().map(|err| err.to_string());
    }

    fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.dark = dark;
        Theme::change(
            if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            None,
            cx,
        );
        palette::apply_theme_overrides(cx);
        self.persist();
        cx.notify();
    }

    fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.collapsed = collapsed;
        self.persist();
        cx.notify();
    }

    /// Switch the native window between the frosted (acrylic) backdrop and an
    /// opaque one; the palette follows instantly because every layer reads
    /// this flag per frame.
    fn set_translucent(&mut self, translucent: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.translucent = translucent;
        window.set_background_appearance(if translucent {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        });
        self.persist();
        cx.notify();
    }

    /// Wrap a freshly mounted page in a short fade-and-rise entrance. The
    /// element state for a page id is dropped while that page is hidden, so
    /// every navigation replays the motion.
    fn enter_page<E>(&self, page: Page, el: E) -> AnyElement
    where
        E: gpui_kit::Styled + IntoElement + 'static,
    {
        if !self.animations {
            return el.into_any_element();
        }
        el.with_animation(
            SharedString::from(format!("page-enter-{}", page.index())),
            Animation::new(Duration::from_millis(180)).with_easing(ease_out_quint()),
            move |el, delta| el.opacity(0.55 + 0.45 * delta).mt(px(10. * (1. - delta))),
        )
        .into_any_element()
    }

    fn menu_item(
        &self,
        id: &'static str,
        label: &'static str,
        icon: IconName,
        page: Page,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.page == page;
        let collapsed = self.collapsed;
        h_flex()
            .id(id)
            .w_full()
            .h(px(36.))
            .items_center()
            .cursor_pointer()
            .rounded_lg()
            .bg(rgb(if active { p.selected } else { p.sidebar }))
            .text_color(rgb(if active { p.text } else { p.muted }))
            .when(!collapsed, |el| el.gap_3().px_2())
            .when(collapsed, |el| el.justify_center())
            .when(!collapsed, |el| {
                el.child(div().w(px(3.)).h(px(16.)).rounded_full().bg(if active {
                    rgb(p.accent_ink)
                } else {
                    rgb(p.sidebar)
                }))
            })
            .child(
                div()
                    .w(px(20.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon).text_color(rgb(if active {
                        p.accent_ink
                    } else {
                        p.muted
                    }))),
            )
            .when(!self.collapsed, |el| {
                el.child(
                    div()
                        .text_sm()
                        .when(active, |label| label.font_medium())
                        .child(label),
                )
            })
            .hover(move |s| s.bg(rgb(p.hover)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.page = page;
                cx.notify();
            }))
            .test_support()
            .aria_label(label)
            .aria_selected(active)
    }

    fn sidebar(&self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let collapsed = self.collapsed;
        let base = v_flex()
            .id("sidebar")
            .test_support()
            .aria_label(if collapsed {
                "Navigation rail"
            } else {
                "Navigation sidebar"
            })
            .h_full()
            .w(px(if collapsed { 64. } else { 232. }))
            .flex_shrink_0()
            .gap_2()
            .p_3()
            .bg(kit::rgba(
                p.sidebar,
                if self.translucent {
                    p.alpha_sidebar
                } else {
                    1.0
                },
            ))
            .border_r_1()
            .border_color(rgb(p.border))
            .child(
                h_flex()
                    .h(px(56.))
                    .items_center()
                    .when(!collapsed, |el| el.gap_3().px_2())
                    .when(collapsed, |el| el.justify_center())
                    .child(
                        div()
                            .flex_shrink_0()
                            .size_8()
                            .rounded_lg()
                            .bg(rgb(p.logo))
                            .text_color(rgb(p.logo_text))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_semibold()
                            .child("G"),
                    )
                    .when(!collapsed, |el| {
                        el.child(
                            v_flex()
                                .child(div().text_sm().font_semibold().child("Workbench"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(p.subtle))
                                        .child("GPUI KIT / NATIVE"),
                                ),
                        )
                    }),
            )
            .child(div().h(px(6.)))
            .when(!collapsed, |el| {
                el.child(
                    div()
                        .px_3()
                        .pb_1()
                        .text_xs()
                        .text_color(rgb(p.subtle))
                        .child("NAVIGATION"),
                )
            })
            .child(self.menu_item(
                "nav-overview",
                "Home",
                IconName::LayoutDashboard,
                Page::Overview,
                p,
                cx,
            ))
            .child(self.menu_item(
                "nav-components",
                "Library",
                IconName::GalleryVerticalEnd,
                Page::Components,
                p,
                cx,
            ))
            .child(self.menu_item(
                "nav-activity",
                "Tasks",
                IconName::Inbox,
                Page::Activity,
                p,
                cx,
            ))
            .child(div().h(px(10.)))
            .when(!self.collapsed, |el| {
                el.child(
                    div()
                        .px_3()
                        .pb_1()
                        .text_xs()
                        .text_color(rgb(p.subtle))
                        .child("SYSTEM"),
                )
            })
            .child(self.menu_item(
                "nav-settings",
                "Preferences",
                IconName::Settings,
                Page::Settings,
                p,
                cx,
            ))
            .child(self.menu_item(
                "nav-developer",
                "Developer",
                IconName::Cpu,
                Page::Developer,
                p,
                cx,
            ))
            .child(div().flex_1())
            .child(kit::divider(p).mx_1())
            .child(
                h_flex()
                    .id("sidebar-toggle")
                    .h(px(38.))
                    .gap_2()
                    .when(!collapsed, |el| el.px_3())
                    .when(collapsed, |el| el.justify_center())
                    .items_center()
                    .rounded_lg()
                    .cursor_pointer()
                    .text_color(rgb(p.muted))
                    .hover(move |s| s.bg(rgb(p.hover)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.set_collapsed(!this.collapsed, cx);
                    }))
                    .child(Icon::new(if self.collapsed {
                        IconName::PanelLeftOpen
                    } else {
                        IconName::PanelLeftClose
                    }))
                    .when(!collapsed, |el| {
                        el.child(div().text_sm().child("Collapse sidebar"))
                    })
                    .test_support()
                    .aria_label(if self.collapsed {
                        "Expand sidebar"
                    } else {
                        "Collapse sidebar"
                    }),
            )
            .child(div().h(px(2.)))
            .child(
                h_flex()
                    .h(px(48.))
                    .gap_3()
                    .items_center()
                    .when(!collapsed, |el| el.px_1())
                    .when(collapsed, |el| el.justify_center())
                    .child(
                        div()
                            .size_8()
                            .rounded_full()
                            .bg(rgb(p.avatar))
                            .text_color(rgb(p.text))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .font_semibold()
                            .child("W"),
                    )
                    .when(!collapsed, |el| {
                        el.child(
                            v_flex()
                                .child(div().text_sm().font_medium().child("On this device"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(p.subtle))
                                        .child("No account needed"),
                                ),
                        )
                    }),
            );

        if !self.animations {
            return base.into_any_element();
        }

        // The animation id carries the target state, so every collapse or
        // expand restarts the transition from the current width.
        let from = px(if collapsed { 232. } else { 64. });
        let to = px(if collapsed { 64. } else { 232. });
        base.with_animation(
            SharedString::from(format!("sidebar-width-{collapsed}")),
            Animation::new(Duration::from_millis(200)).with_easing(ease_in_out),
            move |el, delta| el.w(from + (to - from) * delta),
        )
        .into_any_element()
    }

    fn header(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .h(px(60.))
            .flex_shrink_0()
            .px_7()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(rgb(p.border))
            .bg(rgb(p.surface))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .size_7()
                            .rounded_md()
                            .bg(rgb(p.accent_soft))
                            .text_color(rgb(p.accent_ink))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Icon::new(self.page.icon()).small()),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().text_color(rgb(p.subtle)).child("Workbench"))
                            .child(div().text_sm().text_color(rgb(p.subtle)).child("/"))
                            .child(div().text_sm().font_semibold().child(self.page.title())),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .h(px(22.))
                            .rounded_full()
                            .border_1()
                            .border_color(rgb(p.border))
                            .bg(rgb(p.surface))
                            .child(div().size_1().rounded_full().bg(rgb(p.status_text)))
                            .child(div().text_xs().text_color(rgb(p.muted)).child("LOCAL ONLY")),
                    )
                    .child(
                        Button::new("theme-button")
                            .ghost()
                            .icon(if self.dark {
                                IconName::Sun
                            } else {
                                IconName::Moon
                            })
                            .tooltip(if self.dark {
                                "Switch to light appearance"
                            } else {
                                "Switch to dark appearance"
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_dark(!this.dark, cx);
                            })),
                    ),
            )
    }

    fn page_hero(&self, p: Palette) -> impl IntoElement {
        v_flex()
            .gap_2()
            .pt_2()
            .child(
                div()
                    .id("page-eyebrow")
                    .test_support()
                    .aria_label(self.page.eyebrow())
                    .text_xs()
                    .font_semibold()
                    .text_color(rgb(p.accent_ink))
                    .child(self.page.eyebrow()),
            )
            .child(
                div()
                    .id("page-title")
                    .test_support()
                    .aria_label(self.page.title_leadin())
                    .text_2xl()
                    .font_semibold()
                    .child(self.page.title_leadin()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(p.muted))
                    .max_w(px(640.))
                    .child(self.page.lead()),
            )
    }

    fn destination_row(
        &self,
        destination: Destination,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Destination {
            id,
            title,
            description,
            icon,
            page,
        } = destination;
        let hovered = self.hovered_destination == Some(id);
        h_flex()
            .id(id)
            .w_full()
            .min_w_0()
            .gap_4()
            .px_5()
            .py_4()
            .items_center()
            .rounded_lg()
            .cursor_pointer()
            .bg(rgb(p.surface))
            .border_1()
            .border_color(rgb(p.border))
            .hover(move |s| s.bg(rgb(p.hover)))
            .on_hover(cx.listener(move |this, is_hovered: &bool, _, cx| {
                // Clear only if this row is still the tracked one, so sliding
                // between adjacent rows never wipes the state out of order.
                if *is_hovered {
                    if this.hovered_destination != Some(id) {
                        this.hovered_destination = Some(id);
                        cx.notify();
                    }
                } else if this.hovered_destination == Some(id) {
                    this.hovered_destination = None;
                    cx.notify();
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.page = page;
                cx.notify();
            }))
            .child(
                div()
                    .size_9()
                    .rounded_md()
                    .bg(rgb(p.accent_soft))
                    .text_color(rgb(p.accent_ink))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(div().text_sm().font_medium().child(title))
                    .child(div().text_xs().text_color(rgb(p.muted)).child(description)),
            )
            .child(
                Icon::new(IconName::ArrowRight)
                    .when(hovered, |arrow| {
                        arrow.ml(px(2.)).text_color(rgb(p.accent_ink))
                    })
                    .text_color(rgb(p.subtle)),
            )
            .test_support()
            .aria_label(title)
    }

    fn overview(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let capacity = (self.tasks.len() as f32 / MAX_TASKS as f32) * 100.;
        let task_count = self.tasks.len();
        let done_count = self.tasks.iter().filter(|task| task.done).count();
        let done_pct = if task_count == 0 {
            0.
        } else {
            done_count as f32 / task_count as f32 * 100.
        };
        let count_up: AnyElement = if self.animations {
            div()
                .id("overview-task-count")
                .test_support()
                .aria_label(SharedString::from(format!("{task_count}")))
                .text_3xl()
                .font_semibold()
                .with_animation(
                    // Id changes with the value so every task change replays.
                    SharedString::from(format!("task-count-{task_count}")),
                    Animation::new(Duration::from_millis(400)).with_easing(ease_out_quint()),
                    move |el, delta| {
                        el.child(SharedString::from(format!(
                            "{}",
                            (task_count as f32 * delta).round() as usize
                        )))
                    },
                )
                .into_any_element()
        } else {
            div()
                .id("overview-task-count")
                .test_support()
                .aria_label(SharedString::from(format!("{task_count}")))
                .text_3xl()
                .font_semibold()
                .child(SharedString::from(format!("{task_count}")))
                .into_any_element()
        };
        v_flex()
            .gap_5()
            .child(self.page_hero(p))
            .child(
                h_flex()
                    .gap_5()
                    .items_stretch()
                    .child(
                        card(p)
                            .p_5()
                            .w(px(600.))
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(rgb(p.subtle))
                                            .child("YOUR TASKS"),
                                    )
                                    .child(div().flex_1())
                                    .child(div().text_xs().text_color(rgb(p.muted)).child(
                                        SharedString::from(format!(
                                            "{done_count} done · {} / {MAX_TASKS} stored",
                                            self.tasks.len()
                                        )),
                                    )),
                            )
                            .child(
                                h_flex()
                                    .gap_4()
                                    .items_center()
                                    .child(count_up)
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .min_w_0()
                                            .child(div().text_sm().text_color(rgb(p.muted)).child(
                                                SharedString::from(format!(
                                                    "{} of {} done",
                                                    done_count, task_count
                                                )),
                                            ))
                                            .child(
                                                gpui_kit::component::progress::Progress::new(
                                                    "overview-progress",
                                                )
                                                .value(done_pct)
                                                .w(px(180.)),
                                            ),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        gpui_kit::component::progress::ProgressCircle::new(
                                            "overview-capacity",
                                        )
                                        .value(capacity),
                                    ),
                            )
                            .child(kit::divider(p))
                            .when(self.tasks.is_empty(), |el| {
                                el.child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(p.muted))
                                        .child("No tasks yet — create one from the Tasks page."),
                                )
                            })
                            .children(self.tasks.iter().rev().take(3).map(|task| {
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .min_w_0()
                                    .child(if task.done {
                                        Icon::new(IconName::CircleCheck)
                                            .text_color(rgb(p.accent_ink))
                                            .into_any_element()
                                    } else {
                                        div()
                                            .size_2()
                                            .flex_shrink_0()
                                            .rounded_full()
                                            .border_1()
                                            .border_color(rgb(p.border))
                                            .into_any_element()
                                    })
                                    .child(
                                        div()
                                            .text_sm()
                                            .min_w_0()
                                            .truncate()
                                            .text_color(rgb(if task.done {
                                                p.subtle
                                            } else {
                                                p.text
                                            }))
                                            .when(task.done, |el| el.line_through())
                                            .child(SharedString::from(task.title.clone())),
                                    )
                            }))
                            .when(self.tasks.len() > 3, |el| {
                                el.child(div().text_xs().text_color(rgb(p.subtle)).child(
                                    SharedString::from(format!(
                                        "{} more on the task list",
                                        self.tasks.len() - 3
                                    )),
                                ))
                            })
                            .child(div().flex_1())
                            .child(
                                h_flex().items_center().child(
                                    Button::new("overview-activity")
                                        .primary()
                                        .label("Open task list")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.page = Page::Activity;
                                            cx.notify();
                                        })),
                                ),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .child("Explore the workspace"),
                            )
                            .child(self.destination_row(
                                Destination {
                                    id: "overview-components",
                                    title: "Design kit",
                                    description: "Buttons, inputs, selection, indicators, feedback and color roles",
                                    icon: IconName::GalleryVerticalEnd,
                                    page: Page::Components,
                                },
                                p,
                                cx,
                            ))
                            .child(self.destination_row(
                                Destination {
                                    id: "overview-preferences",
                                    title: "Appearance & layout",
                                    description: "Theme, motion, frosted glass and navigation",
                                    icon: IconName::Settings,
                                    page: Page::Settings,
                                },
                                p,
                                cx,
                            )),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(p.subtle))
                    .child("A native desktop reference project. Everything runs locally."),
            )
    }

    fn activity(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let row_pad = if self.density == 1 { px(8.) } else { px(14.) };
        let done_count = self.tasks.iter().filter(|task| task.done).count();
        v_flex()
            .gap_5()
            .child(self.page_hero(p))
            .child(self.task_editor(p, cx))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .id("task-summary")
                            .test_support()
                            .aria_label(SharedString::from(format!(
                                "{} saved, {} done",
                                self.tasks.len(),
                                done_count
                            )))
                            .text_sm()
                            .font_semibold()
                            .child(SharedString::from(format!(
                                "Saved tasks · {} ({} done)",
                                self.tasks.len(),
                                done_count
                            ))),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .id("task-density")
                            .test_support()
                            .aria_label(if self.density == 1 {
                                "Compact density"
                            } else {
                                "Comfortable density"
                            })
                            .text_xs()
                            .text_color(rgb(p.subtle))
                            .child(if self.density == 1 {
                                "COMPACT DENSITY"
                            } else {
                                "COMFORTABLE DENSITY"
                            }),
                    ),
            )
            .child(
                card(p)
                    .overflow_hidden()
                    .when(self.tasks.is_empty(), |el| {
                        el.child(
                            v_flex()
                                .id("task-empty")
                                .test_support()
                                .aria_label("No tasks yet")
                                .gap_3()
                                .p_12()
                                .items_center()
                                .child(
                                    div()
                                        .size_12()
                                        .rounded_full()
                                        .bg(rgb(p.accent_soft))
                                        .text_color(rgb(p.accent_ink))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(Icon::new(IconName::Inbox).large()),
                                )
                                .child(div().text_sm().font_medium().child("Nothing here yet"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(p.muted))
                                        .child("Create a task above to populate this list."),
                                ),
                        )
                    })
                    .children(self.tasks.iter().rev().enumerate().map(|(i, task)| {
                        let index = self.tasks.len() - 1 - i;
                        let done = task.done;
                        h_flex()
                            .id(SharedString::from(format!("task-row-{index}")))
                            .gap_3()
                            .px_5()
                            .py(row_pad)
                            .items_center()
                            .when(index != self.tasks.len() - 1, |el| {
                                el.border_t_1().border_color(rgb(p.border))
                            })
                            .hover(move |s| s.bg(rgb(p.hover)))
                            .test_support()
                            .aria_label(SharedString::from(format!("Task: {}", task.title)))
                            .child(
                                Checkbox::new(SharedString::from(format!("task-done-{index}")))
                                    .checked(done)
                                    .on_change(cx.listener(move |this, checked, _, cx| {
                                        if let Some(entry) = this.tasks.get_mut(index) {
                                            entry.done = *checked;
                                            this.persist();
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .min_w_0()
                                    .truncate()
                                    .text_color(rgb(if done { p.subtle } else { p.text }))
                                    .when(done, |el| el.line_through())
                                    .child(SharedString::from(task.title.clone())),
                            )
                            .child(div().flex_1())
                            .child(
                                Button::new(SharedString::from(format!("remove-task-{index}")))
                                    .ghost()
                                    .small()
                                    .icon(IconName::Delete)
                                    .tooltip("Remove this task")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if index < this.tasks.len() {
                                            this.tasks.remove(index);
                                            this.persist();
                                            cx.notify();
                                        }
                                    })),
                            )
                    })),
            )
    }

    fn settings(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let storage_path = storage::settings_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "Unavailable on this system".to_string());
        v_flex()
            .gap_5()
            .child(self.page_hero(p))
            .child(
                card(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(div().text_sm().font_semibold().child("Settings"))
                    .child(setting_row(
                        "row-appearance",
                        IconName::Moon,
                        "Dark appearance",
                        "Switch the workbench between light and dark palettes.",
                        p,
                        Switch::new("dark-switch")
                            .checked(self.dark)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.set_dark(*checked, cx);
                            })),
                    ))
                    .child(divider(p))
                    .child(setting_row(
                        "row-navigation",
                        IconName::PanelLeft,
                        "Collapsed sidebar",
                        "Keep the navigation rail collapsed to icons only.",
                        p,
                        Switch::new("sidebar-switch")
                            .checked(self.collapsed)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.set_collapsed(*checked, cx);
                            })),
                    ))
                    .child(divider(p))
                    .child(setting_row(
                        "row-motion",
                        IconName::Play,
                        "Interface motion",
                        "Animate the sidebar, page changes and counters.",
                        p,
                        Switch::new("animations-switch")
                            .checked(self.animations)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.animations = *checked;
                                this.persist();
                                cx.notify();
                            })),
                    ))
                    .child(divider(p))
                    .child(setting_row(
                        "row-frosted",
                        IconName::Eye,
                        "Frosted background",
                        "Let the native acrylic / vibrancy blur show through the workbench.",
                        p,
                        Switch::new("translucent-switch")
                            .checked(self.translucent)
                            .on_change(cx.listener(|this, checked, window, cx| {
                                this.set_translucent(*checked, window, cx);
                            })),
                    )),
            )
            .child(
                card(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_sm().font_semibold().child("Storage"))
                    .child(
                        h_flex()
                            .id("storage-path-row")
                            .gap_3()
                            .items_center()
                            .child(Icon::new(IconName::HardDrive).text_color(rgb(p.subtle)))
                            .child(
                                div()
                                    .id("storage-path")
                                    .test_support()
                                    .aria_label(SharedString::from(storage_path.clone()))
                                    .text_sm()
                                    .text_color(rgb(p.muted))
                                    .child(SharedString::from(storage_path)),
                            ),
                    )
                    .child(divider(p))
                    .child(
                        h_flex().gap_3().items_center().child(
                            div()
                                .id("app-version")
                                .test_support()
                                .aria_label(SharedString::from(format!(
                                    "GPUI Kit Demo {}",
                                    env!("CARGO_PKG_VERSION")
                                )))
                                .text_sm()
                                .text_color(rgb(p.muted))
                                .child(SharedString::from(format!(
                                    "GPUI Kit Demo {} · Rust · GPUI Kit 0.9.0",
                                    env!("CARGO_PKG_VERSION")
                                ))),
                        ),
                    )
                    .child(div().text_xs().text_color(rgb(p.subtle)).child(
                        "Appearance, sidebar and tasks are saved locally between sessions.",
                    )),
            )
    }

    /// The Developer page: runtime facts, the stored state, and the debugging
    /// affordances this project ships with (inspector, resets, demo triggers).
    fn developer(&self, p: Palette, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let viewport = window.viewport_size();
        let scale = window.scale_factor();
        let storage_path = storage::settings_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "Unavailable on this system".to_string());
        let storage_size = storage::settings_size()
            .map(format_bytes)
            .unwrap_or_else(|| "not written yet".to_string());
        let settings_json = storage::to_pretty_json(&self.current_settings());
        let json_preview: String = settings_json.chars().take(900).collect();
        let uptime = self.started.elapsed().as_secs_f32();

        v_flex()
            .gap_5()
            .child(self.page_hero(p))
            .child(
                card(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(div().text_sm().font_semibold().child("Runtime"))
                    .child(dev_row(
                        "Version",
                        format!("GPUI Kit Demo {}", env!("CARGO_PKG_VERSION")),
                        p,
                    ))
                    .child(dev_row(
                        "Build",
                        if cfg!(debug_assertions) {
                            "debug · element inspector available".to_string()
                        } else {
                            "release".to_string()
                        },
                        p,
                    ))
                    .child(dev_row(
                        "Platform",
                        format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
                        p,
                    ))
                    .child(dev_row(
                        "Viewport",
                        format!(
                            "{} x {} @ {}x",
                            viewport.width.as_f32().round(),
                            viewport.height.as_f32().round(),
                            scale
                        ),
                        p,
                    ))
                    .child(divider(p))
                    .child(dev_row("Frames rendered", self.renders.to_string(), p))
                    .child(dev_row("Uptime", format!("{uptime:.1}s"), p))
                    .child(dev_row("Recorded clicks", self.clicks.to_string(), p))
                    .child(dev_row("Completed saves", self.saves_done.to_string(), p)),
            )
            .child(
                card(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_sm().font_semibold().child("Storage"))
                    .child(
                        div()
                            .id("dev-storage-path")
                            .test_support()
                            .aria_label(SharedString::from(storage_path.clone()))
                            .text_xs()
                            .text_color(rgb(p.muted))
                            .child(SharedString::from(storage_path)),
                    )
                    .child(dev_row("Settings file", storage_size, p))
                    .child(divider(p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .child(
                                Button::new("dev-open-folder")
                                    .label("Reveal in file manager")
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        reveal_storage_folder();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("dev-reset-tasks")
                                    .when(self.armed_reset == Some("tasks"), |button| {
                                        button.danger().label("Click again: clear tasks")
                                    })
                                    .when(self.armed_reset != Some("tasks"), |button| {
                                        button.label("Clear tasks")
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.armed_reset == Some("tasks") {
                                            this.tasks.clear();
                                            this.armed_reset = None;
                                            this.persist();
                                        } else {
                                            this.armed_reset = Some("tasks");
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("dev-reset-preferences")
                                    .when(self.armed_reset == Some("prefs"), |button| {
                                        button.danger().label("Click again: reset preferences")
                                    })
                                    .when(self.armed_reset != Some("prefs"), |button| {
                                        button.label("Reset preferences")
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.armed_reset == Some("prefs") {
                                            this.armed_reset = None;
                                            this.reset_preferences(cx);
                                        } else {
                                            this.armed_reset = Some("prefs");
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("dev-clear-all")
                                    .when(self.armed_reset == Some("all"), |button| {
                                        button.danger().label("Click again: delete everything")
                                    })
                                    .when(self.armed_reset != Some("all"), |button| {
                                        button.label("Clear everything")
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.armed_reset == Some("all") {
                                            this.armed_reset = None;
                                            this.tasks.clear();
                                            this.reset_preferences(cx);
                                            this.remove_store_file();
                                        } else {
                                            this.armed_reset = Some("all");
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(caption(
                        "Destructive actions ask for a second click, so a stray press \
                         cannot lose data.",
                        p,
                    )),
            )
            .child(
                card(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_sm().font_semibold().child("State"))
                    .child(
                        v_flex()
                            .id("dev-state-json")
                            .test_support()
                            .aria_label(SharedString::from(json_preview.clone()))
                            .gap_1()
                            .p_3()
                            .rounded_lg()
                            .bg(rgb(p.canvas))
                            .border_1()
                            .border_color(rgb(p.border))
                            .children(
                                json_preview
                                    .lines()
                                    .map(|line| {
                                        div()
                                            .text_xs()
                                            .font_family("monospace")
                                            .text_color(rgb(p.muted))
                                            .child(SharedString::from(line.to_string()))
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(
                                Button::new("dev-copy-state")
                                    .label("Copy as JSON")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let json =
                                            storage::to_pretty_json(&this.current_settings());
                                        cx.write_to_clipboard(ClipboardItem::new_string(json));
                                        this.copied_state = true;
                                        cx.notify();
                                    })),
                            )
                            .when(self.copied_state, |row| {
                                row.child(
                                    div()
                                        .id("dev-copied")
                                        .test_support()
                                        .aria_label("Copied to clipboard")
                                        .text_xs()
                                        .text_color(rgb(p.accent_ink))
                                        .child("Copied to clipboard"),
                                )
                            }),
                    ),
            )
            .child(
                card(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(div().text_sm().font_semibold().child("Debugging tools"))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(p.muted))
                                    .child("Element inspector"),
                            )
                            .child(pill("CTRL + SHIFT + I", p))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(p.subtle))
                                    .child("Hover the live UI to read bounds, roles and state."),
                            ),
                    )
                    .when(cfg!(debug_assertions), |column| {
                        column.child(
                            Button::new("dev-inspector")
                                .label("Toggle the inspector")
                                .on_click(cx.listener(|_, _, window, cx| {
                                    toggle_inspector(window, cx);
                                    cx.notify();
                                })),
                        )
                    })
                    .child(divider(p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(p.muted))
                                    .child("Demo triggers"),
                            )
                            .child(Button::new("dev-busy").label("Run a busy cycle").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.start_save_simulation(cx);
                                }),
                            ))
                            .child(
                                Button::new("dev-toggle-theme")
                                    .label("Flip the theme")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_dark(!this.dark, cx);
                                    })),
                            )
                            .child(
                                Button::new("dev-toggle-rail")
                                    .label("Collapse the rail")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_collapsed(!this.collapsed, cx);
                                    })),
                            ),
                    )
                    .child(caption(
                        "Every trigger writes the same state the screens render, so the \
                         counters above move with it.",
                        p,
                    )),
            )
    }

    /// Restore the default preferences. Stored tasks are left alone; the
    /// Developer page has its own button for those.
    fn reset_preferences(&mut self, cx: &mut Context<Self>) {
        self.dark = false;
        self.collapsed = false;
        self.animations = true;
        self.translucent = true;
        self.copied_state = false;
        Theme::change(ThemeMode::Light, None, cx);
        palette::apply_theme_overrides(cx);
        self.persist();
    }

    /// Delete the settings file, honouring the test store override.
    fn remove_store_file(&mut self) {
        let path = self.store.clone().or_else(|| storage::settings_path().ok());
        if let Some(path) = path {
            let _ = std::fs::remove_file(path);
        }
    }

    fn task_editor(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        card(p)
            .p_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(div().text_sm().font_medium().child("New task"))
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div().min_w_0().flex_1().child(
                            Input::new(&self.input)
                                .id("task-input")
                                .aria_label("New task title"),
                        ),
                    )
                    .child(
                        Button::new("add-task")
                            .primary()
                            .label("Add task")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let entered = this.input.read(cx).value().to_string();
                                if !entered.trim().is_empty()
                                    && this.tasks.len() < storage::MAX_TASKS
                                {
                                    let title = entered
                                        .trim()
                                        .chars()
                                        .take(storage::MAX_TASK_CHARS)
                                        .collect::<String>();
                                    this.tasks.push(storage::TaskEntry::new(title));
                                    this.input
                                        .update(cx, |input, cx| input.set_value("", window, cx));
                                    this.persist();
                                    cx.notify();
                                }
                            })),
                    ),
            )
            .child(
                div()
                    .id("task-editor-note")
                    .test_support()
                    .aria_label(SharedString::from(format!(
                        "{} of {} task slots used",
                        self.tasks.len(),
                        storage::MAX_TASKS
                    )))
                    .text_xs()
                    .text_color(rgb(p.subtle))
                    .child(SharedString::from(format!(
                        "Saved automatically · {} / {} tasks",
                        self.tasks.len(),
                        storage::MAX_TASKS
                    ))),
            )
    }

    fn status_bar(&self, p: Palette) -> impl IntoElement {
        let errored = self.save_error.is_some();
        h_flex()
            .id("status-bar")
            .test_support()
            .aria_label(SharedString::from(
                self.save_error
                    .as_ref()
                    .map(|err| format!("Save error: {err}"))
                    .unwrap_or_else(|| "Local storage ready".to_string()),
            ))
            .w_full()
            .h(px(32.))
            .flex_shrink_0()
            .px_7()
            .items_center()
            .justify_between()
            .border_t_1()
            .border_color(rgb(p.border))
            .bg(rgb(p.surface))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(p.subtle))
                    .child("GPUI KIT · NATIVE DESKTOP"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .h(px(20.))
                    .px_2()
                    .rounded_full()
                    .bg(if errored {
                        rgb(p.accent_soft)
                    } else {
                        rgb(p.status_bg)
                    })
                    .child(div().size_1().rounded_full().bg(rgb(if errored {
                        p.accent_ink
                    } else {
                        p.status_text
                    })))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(if errored { p.accent_ink } else { p.status_text }))
                            .child(SharedString::from(
                                self.save_error
                                    .as_ref()
                                    .map(|err| format!("Save error: {err}"))
                                    .unwrap_or_else(|| "Local storage ready".to_string()),
                            )),
                    ),
            )
    }
}

impl Page {
    fn title_leadin(self) -> &'static str {
        match self {
            Self::Overview => "Workspace overview",
            Self::Components => "Design kit",
            Self::Activity => "Tasks",
            Self::Settings => "Appearance & layout",
            Self::Developer => "Developer tools",
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.renders += 1;
        let p = Palette::new(self.dark);
        // Actions are registered here on purpose: element-level handlers may
        // only be attached during paint, where this render runs every frame.
        h_flex()
            .id("workspace-root")
            // Register before `track_focus` so the focus binding is observable.
            .test_support()
            .aria_label(self.page.title())
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &NavigateHome, _, cx| {
                this.page = Page::Overview;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NavigateLibrary, _, cx| {
                this.page = Page::Components;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NavigateTasks, _, cx| {
                this.page = Page::Activity;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NavigatePreferences, _, cx| {
                this.page = Page::Settings;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NavigateDeveloper, _, cx| {
                this.page = Page::Developer;
                this.armed_reset = None;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &FocusNewTask, window, cx| {
                this.page = Page::Activity;
                let handle = this.input.read(cx).focus_handle(cx);
                window.focus(&handle, cx);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.set_collapsed(!this.collapsed, cx);
            }))
            .size_full()
            .text_color(rgb(p.text))
            .child(self.sidebar(p, cx))
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .bg(kit::rgba(
                        p.canvas,
                        if self.translucent {
                            p.alpha_canvas
                        } else {
                            1.0
                        },
                    ))
                    .child(self.header(p, cx))
                    // The scroll region is wrapped in an identified element:
                    // the scrollable itself consumes its child's id, and this
                    // wrapper keeps a stable, always-visible target for tests
                    // and the element inspector.
                    .child(
                        div()
                            .id("workspace-scroll")
                            .test_support()
                            .aria_label("Page content")
                            .flex_1()
                            .min_h_0()
                            .child(
                                v_flex()
                                    .size_full()
                                    .items_center()
                                    .overflow_y_scrollbar()
                                    .child(v_flex().w_full().max_w(px(1050.)).p_7().child(
                                        match self.page {
                                            Page::Overview => {
                                                self.enter_page(self.page, self.overview(p, cx))
                                            }
                                            Page::Components => {
                                                self.enter_page(self.page, self.library(p, cx))
                                            }
                                            Page::Activity => {
                                                self.enter_page(self.page, self.activity(p, cx))
                                            }
                                            Page::Settings => {
                                                self.enter_page(self.page, self.settings(p, cx))
                                            }
                                            Page::Developer => self.enter_page(
                                                self.page,
                                                self.developer(p, window, cx),
                                            ),
                                        },
                                    )),
                            ),
                    )
                    .child(self.status_bar(p)),
            )
    }
}

/// Mount the workbench: restore local settings, apply the palette and open the
/// main window. Called once by the binary.
pub fn run() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        let (settings, save_error) = match storage::load() {
            Ok(settings) => (settings, None),
            Err(err) => {
                eprintln!("Unable to restore local settings: {err}");
                (
                    storage::Settings::default(),
                    Some(format!("Could not restore settings: {err}")),
                )
            }
        };
        init(cx, &settings);
        open_workbench(cx, settings, save_error).expect("Failed to open GPUI Kit Demo");
    });
}

/// Application-level setup shared by the binary and by tests: GPUI Kit
/// components, the palette for `settings`, and every documented keybinding.
pub fn init(cx: &mut App, settings: &storage::Settings) {
    gpui_kit::init(cx);
    cx.bind_keys(static_action_bindings());
    Theme::change(
        if settings.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        None,
        cx,
    );
    palette::apply_theme_overrides(cx);
}

/// Native window configuration derived from the stored preferences, for a
/// caller-chosen `bounds` so tests can open the shell at a fixed size.
pub fn window_options(settings: &storage::Settings, bounds: WindowBounds) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(bounds),
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from("GPUI Kit Demo")),
            ..Default::default()
        }),
        window_min_size: Some(size(px(720.), px(480.))),
        // Native acrylic (Windows) / material (macOS) behind translucent
        // areas. Toggled at runtime from Preferences via
        // `Window::set_background_appearance`.
        window_background: if settings.translucent {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        },
        ..Default::default()
    }
}

/// Open the workbench window with `settings` already restored. Returns the
/// window handle and the mounted [`Workspace`] entity.
pub fn open_workbench(
    cx: &mut App,
    settings: storage::Settings,
    save_error: Option<String>,
) -> gpui_kit::Result<(AnyWindowHandle, Entity<Workspace>)> {
    let bounds = WindowBounds::centered(size(px(1220.), px(800.)), cx);
    let options = window_options(&settings, bounds);
    gpui_kit::open_window(options, cx, move |window, cx| {
        cx.new(|cx| Workspace::new(window, cx, settings, save_error))
    })
}

/// A label/value row on the Developer page.
fn dev_row(label: &'static str, value: String, p: Palette) -> Div {
    h_flex()
        .w_full()
        .gap_4()
        .items_center()
        .justify_between()
        .child(div().text_sm().text_color(rgb(p.muted)).child(label))
        .child(
            div()
                .text_sm()
                .font_medium()
                .child(SharedString::from(value)),
        )
}

/// Human-readable file size for the Developer page.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    match bytes {
        0..KB => format!("{bytes} B"),
        KB..=1048576 => format!("{:.1} KB", bytes as f32 / KB as f32),
        _ => format!("{:.1} MB", bytes as f32 / 1048576.),
    }
}

/// Toggle the element inspector. Debug builds ship it; release builds only
/// keep the button compiled away with the rest of the tooling.
fn toggle_inspector(window: &mut Window, cx: &mut App) {
    #[cfg(debug_assertions)]
    window.toggle_inspector(cx);
    #[cfg(not(debug_assertions))]
    let _ = (window, cx);
}

/// Reveal the settings file in the platform file manager. Best effort: a
/// failure is not worth interrupting the user over.
fn reveal_storage_folder() {
    let path = storage::settings_path().unwrap_or_else(|_| PathBuf::from("."));
    let folder = path.parent().map(PathBuf::from).unwrap_or(path);
    #[cfg(target_os = "windows")]
    let command = "explorer";
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let command = "xdg-open";
    let _ = std::process::Command::new(command).arg(folder).spawn();
}

/// The keybindings documented by the Library page, all live.
pub fn static_action_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("ctrl-1", NavigateHome, None),
        KeyBinding::new("ctrl-2", NavigateLibrary, None),
        KeyBinding::new("ctrl-3", NavigateTasks, None),
        KeyBinding::new("ctrl-4", NavigatePreferences, None),
        KeyBinding::new("ctrl-5", NavigateDeveloper, None),
        KeyBinding::new("ctrl-n", FocusNewTask, None),
        KeyBinding::new("ctrl-b", ToggleSidebar, None),
        KeyBinding::new("ctrl-,", NavigatePreferences, None),
    ]
}

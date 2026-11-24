mod gallery;
mod kit;
mod palette;
mod storage;

use gpui_kit::assets::Assets;
use gpui_kit::base::StyledExt as _;
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
use kit::{card, divider, setting_row};
use palette::Palette;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Overview,
    Components,
    Activity,
    Settings,
}

impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Home",
            Self::Components => "Library",
            Self::Activity => "Tasks",
            Self::Settings => "Preferences",
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Overview => IconName::LayoutDashboard,
            Self::Components => IconName::GalleryVerticalEnd,
            Self::Activity => IconName::Inbox,
            Self::Settings => IconName::Settings,
        }
    }

    fn index(self) -> u8 {
        match self {
            Self::Overview => 0,
            Self::Components => 1,
            Self::Activity => 2,
            Self::Settings => 3,
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::Overview => "WORKBENCH / OVERVIEW",
            Self::Components => "WORKBENCH / COMPONENTS",
            Self::Activity => "WORKBENCH / TASKS",
            Self::Settings => "WORKBENCH / PREFERENCES",
        }
    }

    fn lead(self) -> &'static str {
        match self {
            Self::Overview => "Local tasks, native controls, and appearance settings.",
            Self::Components => "Native interaction patterns and their actual states.",
            Self::Activity => "A small, persistent task list for trying native interactions.",
            Self::Settings => "A small set of durable, device-specific preferences.",
        }
    }
}

struct Workspace {
    page: Page,
    dark: bool,
    collapsed: bool,
    animations: bool,
    translucent: bool,
    clicks: usize,
    tasks: Vec<storage::TaskEntry>,
    input: Entity<InputState>,
    playground_input: Entity<InputState>,
    save_error: Option<String>,
    /// Library demos — real, in-memory state (not persisted by design).
    gallery_busy: bool,
    saves_done: usize,
    show_counts: bool,
    density: usize,
    progress: Entity<SliderState>,
    progress_value: f32,
    /// Focus home of the workbench root; action dispatch rides the focus path.
    focus: FocusHandle,
}

impl Workspace {
    fn new(
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
        }
    }

    fn persist(&mut self) {
        let settings = storage::Settings {
            dark: self.dark,
            collapsed: self.collapsed,
            animations: self.animations,
            translucent: self.translucent,
            tasks: self.tasks.clone(),
        };
        self.save_error = storage::save(&settings).err().map(|err| err.to_string());
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
    }

    fn sidebar(&self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let collapsed = self.collapsed;
        let base = v_flex()
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
                    .text_xs()
                    .font_semibold()
                    .text_color(rgb(p.accent_ink))
                    .child(self.page.eyebrow()),
            )
            .child(
                div()
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
            .child(Icon::new(IconName::ArrowRight).text_color(rgb(p.subtle)))
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
                            .gap_3()
                            .px_5()
                            .py(row_pad)
                            .items_center()
                            .when(index != self.tasks.len() - 1, |el| {
                                el.border_t_1().border_color(rgb(p.border))
                            })
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
                            .gap_3()
                            .items_center()
                            .child(Icon::new(IconName::HardDrive).text_color(rgb(p.subtle)))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(p.muted))
                                    .child(SharedString::from(storage_path)),
                            ),
                    )
                    .child(divider(p))
                    .child(
                        h_flex().gap_3().items_center().child(
                            div()
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
                    .child(div().min_w_0().flex_1().child(Input::new(&self.input)))
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
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = Palette::new(self.dark);
        // Actions are registered here on purpose: element-level handlers may
        // only be attached during paint, where this render runs every frame.
        h_flex()
            .id("workspace-root")
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
                    .child(
                        v_flex()
                            .flex_1()
                            .min_h_0()
                            .items_center()
                            .id("workspace-scroll")
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
                                },
                            )),
                    )
                    .child(self.status_bar(p)),
            )
    }
}

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);

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
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1220.), px(800.)), cx)),
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
        };
        gpui_kit::open_window(options, cx, move |window, cx| {
            cx.new(|cx| Workspace::new(window, cx, settings, save_error))
        })
        .expect("Failed to open GPUI Kit Demo");
    });
}

/// The keybindings documented by the Library page, all live.
fn static_action_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("ctrl-1", NavigateHome, None),
        KeyBinding::new("ctrl-2", NavigateLibrary, None),
        KeyBinding::new("ctrl-3", NavigateTasks, None),
        KeyBinding::new("ctrl-4", NavigatePreferences, None),
        KeyBinding::new("ctrl-n", FocusNewTask, None),
        KeyBinding::new("ctrl-b", ToggleSidebar, None),
        KeyBinding::new("ctrl-,", NavigatePreferences, None),
    ]
}

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
    h_flex,
    input::{Input, InputState},
    switch::Switch,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kit::{card, divider, pill, setting_row};
use palette::Palette;
use storage::MAX_TASKS;

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
    clicks: usize,
    tasks: Vec<String>,
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
        }
    }

    fn persist(&mut self) {
        let settings = storage::Settings {
            dark: self.dark,
            collapsed: self.collapsed,
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
        h_flex()
            .id(id)
            .w_full()
            .h(px(36.))
            .gap_3()
            .px_2()
            .items_center()
            .cursor_pointer()
            .rounded_lg()
            .bg(rgb(if active { p.selected } else { p.sidebar }))
            .text_color(rgb(if active { p.text } else { p.muted }))
            .child(div().w(px(3.)).h(px(16.)).rounded_full().bg(if active {
                rgb(p.accent_ink)
            } else {
                rgb(p.sidebar)
            }))
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
            .when(!self.collapsed, |el| el.child(div().text_sm().child(label)))
            .when(self.collapsed, |el| el.flex().justify_center())
            .hover(move |s| s.bg(rgb(p.hover)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.page = page;
                cx.notify();
            }))
    }

    fn sidebar(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .h_full()
            .w(px(if self.collapsed { 68. } else { 232. }))
            .flex_shrink_0()
            .gap_2()
            .p_3()
            .bg(rgb(p.sidebar))
            .border_r_1()
            .border_color(rgb(p.border))
            .child(
                h_flex()
                    .h(px(56.))
                    .items_center()
                    .gap_3()
                    .when(!self.collapsed, |el| el.px_2())
                    .when(self.collapsed, |el| el.pl_1())
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
                    .when(!self.collapsed, |el| {
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
            .when(!self.collapsed, |el| {
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
                    .when(!self.collapsed, |el| el.px_3())
                    .when(self.collapsed, |el| el.pl_6())
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
                    .when(!self.collapsed, |el| {
                        el.child(div().text_sm().child("Collapse sidebar"))
                    }),
            )
            .child(div().h(px(2.)))
            .child(
                h_flex()
                    .h(px(48.))
                    .px_1()
                    .gap_3()
                    .items_center()
                    .when(self.collapsed, |el| el.justify_center())
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
                    .when(!self.collapsed, |el| {
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
            )
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
                    .child(pill("LOCAL ONLY", p))
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
                    .text_3xl()
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
        id: &'static str,
        title: &'static str,
        description: &'static str,
        icon: IconName,
        page: Page,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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

    fn overview(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let capacity = (self.tasks.len() as f32 / MAX_TASKS as f32) * 100.;
        v_flex().gap_6()
            .child(self.page_hero(p))
            .child(
                h_flex().gap_5().items_stretch()
                    .child(
                        card(p).p_6().flex_1().flex().flex_col().gap_4()
                            .child(
                                h_flex().gap_3().items_center()
                                    .child(
                                        div().text_xs().font_semibold().text_color(rgb(p.subtle))
                                            .child("YOUR TASKS"),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        div().text_xs().text_color(rgb(p.muted))
                                            .child("Stored on this device"),
                                    ),
                            )
                            .child(
                                h_flex().gap_6().items_end()
                                    .child(
                                        div().text_3xl().font_semibold()
                                            .child(SharedString::from(format!("{}", self.tasks.len()))),
                                    )
                                    .child(
                                        v_flex().gap_1().pb_2()
                                            .child(div().text_sm().text_color(rgb(p.muted))
                                                .child(if self.tasks.len() == 1 { "Task in your list" } else { "Tasks in your list" }))
                                            .child(
                                                div().text_xs().text_color(rgb(p.subtle))
                                                    .child(SharedString::from(format!(
                                                        "{} / {} stored",
                                                        self.tasks.len(), MAX_TASKS
                                                    ))),
                                            ),
                                    )
                                    .child(div().flex_1())
                                    .child(
                                        gpui_kit::component::progress::ProgressCircle::new("overview-capacity")
                                            .value(capacity),
                                    ),
                            )
                            .child(h_flex().items_center()
                                .child(
                                    Button::new("overview-activity").primary().label("Open task list")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.page = Page::Activity;
                                            cx.notify();
                                        })),
                                )),
                    )
                    .child(
                        v_flex().flex_1().gap_5()
                            .child(
                                v_flex().gap_2().min_w_0()
                                    .child(div().text_sm().font_semibold().child("Explore the workspace"))
                                    .child(self.destination_row("overview-components", "Design kit",
                                        "Buttons, inputs, selection, indicators, feedback and color roles",
                                        IconName::GalleryVerticalEnd, Page::Components, p, cx)),
                            )
                            .child(
                                v_flex().gap_2().min_w_0()
                                    .child(div().text_sm().font_semibold().child("Adjust the app"))
                                    .child(self.destination_row("overview-preferences", "Appearance & layout",
                                        "Switch themes and adjust the navigation",
                                        IconName::Settings, Page::Settings, p, cx)),
                            ),
                    ),
            )
            .child(
                div().text_xs().text_color(rgb(p.subtle))
                    .child("A native desktop reference project. Everything runs locally."),
            )
    }

    fn activity(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let row_pad = if self.density == 1 { px(8.) } else { px(14.) };
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
                                "Saved tasks · {}",
                                self.tasks.len()
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
                    .children(self.tasks.iter().rev().enumerate().map(|(i, title)| {
                        let index = self.tasks.len() - 1 - i;
                        h_flex()
                            .gap_3()
                            .px_5()
                            .py(row_pad)
                            .items_center()
                            .when(index != self.tasks.len() - 1, |el| {
                                el.border_t_1().border_color(rgb(p.border))
                            })
                            .child(
                                h_flex()
                                    .flex_shrink_0()
                                    .size_7()
                                    .rounded_md()
                                    .bg(rgb(p.canvas))
                                    .text_color(rgb(p.subtle))
                                    .text_xs()
                                    .font_medium()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(SharedString::from(format!("{}", index + 1))),
                            )
                            .child(
                                v_flex().flex_1().gap_1().min_w_0().child(
                                    div()
                                        .text_sm()
                                        .font_medium()
                                        .child(SharedString::from(title.clone())),
                                ),
                            )
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

    fn settings(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let storage_path = storage::settings_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "Unavailable on this system".to_string());
        v_flex()
            .gap_6()
            .child(self.page_hero(p))
            .child(
                card(p)
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(div().text_sm().font_semibold().child("Settings"))
                    .child(setting_row(
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
                        IconName::PanelLeft,
                        "Collapsed sidebar",
                        "Keep the navigation rail collapsed to icons only.",
                        p,
                        Switch::new("sidebar-switch")
                            .checked(self.collapsed)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.set_collapsed(*checked, cx);
                            })),
                    )),
            )
            .child(
                card(p)
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_4()
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
                                .child("GPUI Kit Demo 0.3.0 · Rust · GPUI Kit 0.9.0"),
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
                                    this.tasks.push(title);
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
        h_flex()
            .size_full()
            .bg(rgb(p.canvas))
            .text_color(rgb(p.text))
            .child(self.sidebar(p, cx))
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .min_w_0()
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
                                    Page::Overview => self.overview(p, cx).into_any_element(),
                                    Page::Components => self.library(p, cx).into_any_element(),
                                    Page::Activity => self.activity(p, cx).into_any_element(),
                                    Page::Settings => self.settings(p, cx).into_any_element(),
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
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, move |window, cx| {
            cx.new(|cx| Workspace::new(window, cx, settings, save_error))
        })
        .expect("Failed to open GPUI Kit Demo");
    });
}

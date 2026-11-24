//! Library page: the app's component gallery. Six numbered sections present
//! the native GPUI Kit controls and the workbench's own design kit, every
//! demo wired to real workspace state.

use crate::Workspace;
use crate::kit::{caption, card, divider, field_label, pill, section_head, stage, swatch};
use crate::palette::Palette;
use crate::storage::MAX_TASKS;
use gpui_kit::base::StyledExt as _;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{
    Disableable, Icon, IconName, Sizable,
    badge::Badge,
    button::*,
    checkbox::Checkbox,
    h_flex,
    input::Input,
    kbd::Kbd,
    progress::{Progress, ProgressCircle},
    radio::{Radio, RadioGroup},
    skeleton::Skeleton,
    slider::Slider,
    spinner::Spinner,
    switch::Switch,
    tag::Tag,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{Div, Keystroke, ParentElement, SharedString, Styled, div, px, rgb};
use gpui_kit::{InteractiveElement as _, StatefulInteractiveElement as _};

impl Workspace {
    /// The Library page: a live showcase of every reusable control pattern.
    pub fn library(&self, p: Palette, cx: &mut gpui_kit::Context<Self>) -> Div {
        v_flex()
            .gap_6()
            .child(
                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .id("page-eyebrow")
                            .test_support()
                            .aria_label("COMPONENT LIBRARY")
                            .text_xs()
                            .font_semibold()
                            .text_color(rgb(p.accent_ink))
                            .child("COMPONENT LIBRARY"),
                    )
                    .child(
                        div()
                            .id("page-title")
                            .test_support()
                            .aria_label("Design kit")
                            .text_2xl()
                            .font_semibold()
                            .child("Design kit"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(p.muted))
                            .max_w(px(640.))
                            .child(
                                "Native GPUI Kit controls and workbench patterns, wired to real \
                                 state. Every demo below updates live.",
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(pill("6 SECTIONS", p))
                            .child(pill("GPUI KIT 0.9.0", p))
                            .child(pill(
                                if self.dark {
                                    "DARK PALETTE"
                                } else {
                                    "LIGHT PALETTE"
                                },
                                p,
                            )),
                    ),
            )
            .child(self.actions_card(p, cx))
            .child(self.entry_card(p, cx))
            .child(self.selection_card(p, cx))
            .child(self.indicators_card(p))
            .child(self.feedback_card(p))
            .child(self.colors_card(p))
    }

    /// 01 — Buttons with their real variants, sizes and a busy state.
    fn actions_card(&self, p: Palette, cx: &mut gpui_kit::Context<Self>) -> Div {
        let busy = self.gallery_busy;
        let recorded = self.clicks;
        card(p)
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(section_head(
                "01",
                "Actions",
                "Buttons in every supported role.",
                p,
            ))
            .child(
                stage(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(field_label("VARIANTS", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(
                                Button::new("btn-primary")
                                    .primary()
                                    .label("Primary")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clicks += 1;
                                        cx.notify();
                                    })),
                            )
                            .child(Button::new("btn-secondary").label("Secondary").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.clicks += 1;
                                    cx.notify();
                                }),
                            ))
                            .child(Button::new("btn-ghost").ghost().label("Ghost"))
                            .child(Button::new("btn-danger").danger().label("Danger"))
                            .child(Button::new("btn-link").link().label("Link"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(p.subtle))
                                    .child(SharedString::from(format!("Interactions: {recorded}"))),
                            ),
                    )
                    .child(divider(p))
                    .child(field_label("SIZES AND ICONS", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(Button::new("btn-xs").xsmall().label("Extra small"))
                            .child(Button::new("btn-sm").small().label("Small"))
                            .child(Button::new("btn-md").label("Default"))
                            .child(Button::new("btn-lg").large().label("Large"))
                            .child(
                                Button::new("btn-icon-add")
                                    .ghost()
                                    .icon(IconName::Plus)
                                    .tooltip("Add a task on the Tasks page"),
                            )
                            .child(
                                Button::new("btn-icon-checked")
                                    .ghost()
                                    .icon(IconName::CircleCheck)
                                    .disabled(true),
                            ),
                    )
                    .child(divider(p))
                    .child(field_label("BUSY STATE", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(
                                Button::new("btn-busy")
                                    .primary()
                                    .label("Simulate save")
                                    .loading(busy)
                                    .disabled(busy)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.start_save_simulation(cx);
                                    })),
                            )
                            .child(div().text_xs().text_color(rgb(p.subtle)).child(
                                SharedString::from(format!("Saves completed: {}", self.saves_done)),
                            )),
                    )
                    .child(caption(
                        "Variants, sizes, icons and the busy spinner are all live; nothing \
                             is a static picture.",
                        p,
                    )),
            )
    }

    /// 02 — The controlled text input with a live character readout.
    fn entry_card(&self, p: Palette, cx: &mut gpui_kit::Context<Self>) -> Div {
        let characters = self
            .playground_input
            .read(cx)
            .value()
            .to_string()
            .chars()
            .count();
        card(p)
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(section_head(
                "02",
                "Entry",
                "A controlled text input with live value feedback.",
                p,
            ))
            .child(
                stage(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(field_label("CONTROLLED", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(
                                div().w(px(360.)).child(
                                    Input::new(&self.playground_input)
                                        .id("library-input")
                                        .aria_label("Controlled input"),
                                ),
                            )
                            .when(self.show_counts, |row| {
                                row.child(
                                    pill_dynamic(&format!("Characters: {characters}"), p)
                                        .id("library-characters")
                                        .test_support()
                                        .aria_label(SharedString::from(format!(
                                            "{characters} characters"
                                        ))),
                                )
                            }),
                    )
                    .child(caption(
                        "This input keeps its own InputState; the Tasks editor uses a fully \
                             separate one.",
                        p,
                    )),
            )
    }

    /// 03 — Switch, checkbox and radio, all bound to workspace state.
    fn selection_card(&self, p: Palette, cx: &mut gpui_kit::Context<Self>) -> Div {
        let density = self.density;
        card(p)
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(section_head(
                "03",
                "Selection",
                "Switches, checkboxes and radios bound to real state.",
                p,
            ))
            .child(
                stage(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(field_label("SWITCHES", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_8()
                            .child(
                                Switch::new("gallery-dark-switch")
                                    .label("Dark appearance")
                                    .checked(self.dark)
                                    .on_change(cx.listener(|this, checked, _, cx| {
                                        this.set_dark(*checked, cx);
                                    })),
                            )
                            .child(
                                Switch::new("gallery-collapse-switch")
                                    .label("Collapsed sidebar")
                                    .checked(self.collapsed)
                                    .on_change(cx.listener(|this, checked, _, cx| {
                                        this.set_collapsed(*checked, cx);
                                    })),
                            ),
                    )
                    .child(divider(p))
                    .child(field_label("CHECKBOX", p))
                    .child(
                        Checkbox::new("gallery-count-checkbox")
                            .label("Show character counts in this page")
                            .checked(self.show_counts)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.show_counts = *checked;
                                cx.notify();
                            })),
                    )
                    .child(divider(p))
                    .child(field_label("RADIO GROUP", p))
                    .child(
                        // `RadioGroup` re-keys its children by index, so the
                        // group is wrapped in an identified element: that is
                        // what a test (or the inspector) can point at.
                        div()
                            .id("density-control")
                            .test_support()
                            .aria_label("Row density")
                            .child(
                                RadioGroup::horizontal("density-radio")
                                    .child(
                                        Radio::new("density-comfortable").label("Comfortable rows"),
                                    )
                                    .child(Radio::new("density-compact").label("Compact rows"))
                                    .selected_index(Some(density))
                                    .on_change(cx.listener(|this, index, _, cx| {
                                        this.density = *index;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(caption(
                        "The density choice changes the row padding of the real task list on \
                             the Tasks page.",
                        p,
                    )),
            )
    }

    /// 04 — Badges, tags and keybinding chips.
    fn indicators_card(&self, p: Palette) -> Div {
        let count = self.tasks.len();
        card(p)
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(section_head(
                "04",
                "Indicators",
                "Counts, status tags and keybinding chips.",
                p,
            ))
            .child(
                stage(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(field_label("BADGES", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_5()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(div().text_sm().child("Open tasks"))
                                    .child(Badge::new().count(count).max(MAX_TASKS)),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Icon::new(IconName::Inbox))
                                    .child(Badge::new().dot()),
                            ),
                    )
                    .child(divider(p))
                    .child(field_label("TAG VARIANTS", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(Tag::secondary().child("Secondary"))
                            .child(Tag::primary().child("Primary"))
                            .child(Tag::success().child("Success"))
                            .child(Tag::warning().child("Warning"))
                            .child(Tag::info().child("Info"))
                            .child(Tag::danger().child("Danger")),
                    )
                    .child(divider(p))
                    .child(field_label("APP SHORTCUTS", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(shortcut("Home", "ctrl-1", p))
                            .child(shortcut("Library", "ctrl-2", p))
                            .child(shortcut("Tasks", "ctrl-3", p))
                            .child(shortcut("Preferences", "ctrl-4", p))
                            .child(shortcut("Developer", "ctrl-5", p))
                            .child(shortcut("New task", "ctrl-n", p))
                            .child(shortcut("Sidebar", "ctrl-b", p))
                            .child(shortcut("Inspector", "ctrl-shift-i", p)),
                    )
                    .child(caption(
                        "Every binding above is registered in this build — press one to \
                             try the live navigation, focus and sidebar actions.",
                        p,
                    ))
                    .child(divider(p))
                    .child(field_label("TOOLTIP", p))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .items_center()
                            .child(
                                Button::new("tooltip-demo")
                                    .label("Hover me")
                                    .tooltip("Tooltips appear after a short hover delay"),
                            )
                            .child(
                                Button::new("tooltip-icon-demo")
                                    .ghost()
                                    .icon(IconName::Info)
                                    .tooltip("Icon buttons carry tooltips too"),
                            ),
                    )
                    .child(divider(p))
                    .child(field_label("BUNDLED ICONS", p))
                    .child(
                        h_flex().flex_wrap().gap_2().children(
                            [
                                (IconName::Search, "Search"),
                                (IconName::Star, "Star"),
                                (IconName::Heart, "Heart"),
                                (IconName::Bell, "Bell"),
                                (IconName::Palette, "Palette"),
                                (IconName::Globe, "Globe"),
                                (IconName::Github, "Github"),
                                (IconName::FileText, "File"),
                                (IconName::HardDrive, "Disk"),
                                (IconName::Cpu, "Cpu"),
                                (IconName::Mic, "Mic"),
                                (IconName::Copy, "Copy"),
                                (IconName::Play, "Play"),
                                (IconName::Pause, "Pause"),
                                (IconName::RefreshCw, "Refresh"),
                                (IconName::ExternalLink, "Link"),
                                (IconName::User, "User"),
                                (IconName::Info, "Info"),
                            ]
                            .map(|(icon, name)| {
                                v_flex()
                                    .w(px(64.))
                                    .gap_2()
                                    .items_center()
                                    .py_2()
                                    .rounded_md()
                                    .bg(rgb(p.surface))
                                    .border_1()
                                    .border_color(rgb(p.border))
                                    .child(Icon::new(icon).text_color(rgb(p.accent_ink)))
                                    .child(div().text_xs().text_color(rgb(p.muted)).child(name))
                            }),
                        ),
                    )
                    .child(caption(
                        "A sample of the Lucide set bundled with GPUI Kit assets.",
                        p,
                    )),
            )
    }

    /// 05 — Progress, capacity and loading patterns sharing one slider state.
    fn feedback_card(&self, p: Palette) -> Div {
        let percent = self.progress_value.clamp(0., 100.);
        let capacity = capacity_percent(self.tasks.len());
        card(p)
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(section_head(
                "05",
                "Feedback",
                "Progress, capacity and loading patterns.",
                p,
            ))
            .child(
                stage(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(field_label("PROGRESS AND RANGE", p))
                    .child(
                        v_flex()
                            .w(px(320.))
                            .gap_3()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .child(div().text_sm().child("Slider value"))
                                    .child(
                                        div()
                                            .id("library-slider-value")
                                            .test_support()
                                            .aria_label(SharedString::from(format!(
                                                "{} percent",
                                                percent.round() as i32
                                            )))
                                            .text_sm()
                                            .font_semibold()
                                            .child(SharedString::from(format!(
                                                "{}%",
                                                percent.round() as i32
                                            ))),
                                    ),
                            )
                            .child(
                                div()
                                    .id("library-slider")
                                    .test_support()
                                    .aria_label("Progress slider")
                                    .child(Slider::new(&self.progress).horizontal()),
                            )
                            .child(Progress::new("gallery-progress").value(percent)),
                    )
                    .child(divider(p))
                    .child(field_label("CAPACITY", p))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_center()
                            .child(ProgressCircle::new("gallery-capacity").value(capacity))
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(div().text_sm().child("Stored tasks"))
                                    .child(div().text_xs().text_color(rgb(p.muted)).child(
                                        SharedString::from(format!(
                                            "{} of {} slots used",
                                            self.tasks.len(),
                                            MAX_TASKS
                                        )),
                                    )),
                            ),
                    )
                    .child(divider(p))
                    .child(field_label("LOADING PATTERNS", p))
                    .child(
                        h_flex().gap_5().items_center().child(Spinner::new()).child(
                            v_flex()
                                .w(px(180.))
                                .gap_2()
                                .child(Skeleton::new().h(px(10.)).w_full())
                                .child(Skeleton::new().secondary().h(px(10.)).w(px(120.))),
                        ),
                    )
                    .child(caption(
                        "The slider drives the progress bar and its readout together; the \
                             spinner and skeleton stand for waiting state.",
                        p,
                    )),
            )
    }

    /// 06 — The semantic color roles behind every screen.
    fn colors_card(&self, p: Palette) -> Div {
        let mode = if self.dark { "Dark" } else { "Light" };
        card(p)
            .p_6()
            .flex()
            .flex_col()
            .gap_5()
            .child(section_head(
                "06",
                "Color roles",
                "Semantic tokens for the current appearance.",
                p,
            ))
            .child(
                stage(p)
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .child(swatch("Canvas", p.canvas, p))
                            .child(swatch("Surface", p.surface, p))
                            .child(swatch("Sidebar", p.sidebar, p))
                            .child(swatch("Border", p.border, p))
                            .child(swatch("Hover", p.hover, p))
                            .child(swatch("Selected", p.selected, p)),
                    )
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_3()
                            .child(swatch("Text", p.text, p))
                            .child(swatch("Muted", p.muted, p))
                            .child(swatch("Subtle", p.subtle, p))
                            .child(swatch("Accent soft", p.accent_soft, p))
                            .child(swatch("Accent ink", p.accent_ink, p))
                            .child(swatch("Status", p.status_text, p)),
                    )
                    .child(caption(
                        SharedString::from(format!(
                            "{mode} mode values from src/palette.rs; text and accent pairs \
                                 are contrast-tested."
                        )),
                        p,
                    )),
            )
    }

    /// Real busy flow for the loading demo: one in-flight task, then a count.
    pub(crate) fn start_save_simulation(&mut self, cx: &mut gpui_kit::Context<Self>) {
        use std::time::Duration;

        if self.gallery_busy {
            return;
        }
        self.gallery_busy = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(900))
                .await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    this.gallery_busy = false;
                    this.saves_done += 1;
                    cx.notify();
                });
            }
        })
        .detach();
    }
}

/// Pill accepting formatted text; the static variant lives in `kit`.
fn pill_dynamic(text: &str, p: Palette) -> Div {
    h_flex()
        .items_center()
        .px_2()
        .h(px(22.))
        .rounded_full()
        .border_1()
        .border_color(rgb(p.border))
        .bg(rgb(p.surface))
        .text_xs()
        .text_color(rgb(p.muted))
        .child(text.to_string())
}

fn capacity_percent(len: usize) -> f32 {
    (len as f32 / MAX_TASKS as f32) * 100.
}

/// A labeled keybinding chip; every binding shown here is live.
fn shortcut(label: &'static str, keys: &'static str, p: Palette) -> Div {
    h_flex()
        .gap_2()
        .items_center()
        .child(div().text_xs().text_color(rgb(p.muted)).child(label))
        .child(Kbd::new(Keystroke::parse(keys).unwrap()))
}

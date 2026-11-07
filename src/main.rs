mod palette;

use gpui_kit::assets::Assets;
use gpui_kit::base::StyledExt;
use gpui_kit::component::Sizable;
use gpui_kit::component::{
    button::*, h_flex, input::{Input, InputState}, switch::Switch, v_flex, Icon, IconName,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use palette::Palette;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page { Overview, Components, Activity, Settings }

impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview", Self::Components => "Components",
            Self::Activity => "Activity", Self::Settings => "Settings",
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
}

impl Workspace {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx)
            .placeholder("Describe a local demo task..."));
        Self {
            page: Page::Overview, dark: false, collapsed: false, clicks: 0,
            tasks: vec!["Explore GPUI Kit components".into(), "Customize the desktop shell".into()],
            input,
        }
    }

    fn menu_item(
        &self, id: &'static str, label: &'static str, icon: IconName,
        page: Page, p: Palette, cx: &mut Context<Self>
    ) -> impl IntoElement {
        let active = self.page == page;
        h_flex().id(id).w_full().h(px(38.)).px_3().gap_3()
            .items_center().cursor_pointer().rounded_lg()
            .bg(rgb(if active { p.selected } else { p.sidebar }))
            .text_color(rgb(if active { p.text } else { p.muted }))
            .hover(move |s| s.bg(rgb(p.hover)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.page = page; cx.notify();
            }))
            .child(Icon::new(icon))
            .when(!self.collapsed, |el| el.child(div().text_sm().child(label)))
    }

    fn sidebar(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().h_full().w(px(if self.collapsed { 64. } else { 242. }))
            .flex_shrink_0().gap_2().p_3().bg(rgb(p.sidebar))
            .border_r_1().border_color(rgb(p.border))
            .child(
                h_flex().h(px(56.)).items_center().gap_3().px_2()
                    .child(
                        div().w(px(32.)).h(px(32.)).rounded_lg()
                            .bg(rgb(p.logo)).text_color(rgb(p.logo_text))
                            .flex().items_center().justify_center()
                            .font_semibold().child("G")
                    )
                    .when(!self.collapsed, |el| el.child(
                        v_flex().gap_0()
                            .child(div().text_sm().font_semibold().child("GPUI Studio"))
                            .child(div().text_xs().text_color(rgb(p.subtle)).child("Desktop Starter"))
                    ))
            )
            .child(div().h(px(14.)))
            .when(!self.collapsed, |el| el.child(
                div().px_3().pb_2().text_xs().text_color(rgb(p.subtle))
                    .child("WORKSPACE")
            ))
            .child(self.menu_item("nav-overview", "Overview", IconName::LayoutDashboard, Page::Overview, p, cx))
            .child(self.menu_item("nav-components", "Components", IconName::GalleryVerticalEnd, Page::Components, p, cx))
            .child(self.menu_item("nav-activity", "Activity", IconName::Inbox, Page::Activity, p, cx))
            .child(div().h(px(18.)))
            .when(!self.collapsed, |el| el.child(
                div().px_3().pb_2().text_xs().text_color(rgb(p.subtle))
                    .child("PREFERENCES")
            ))
            .child(self.menu_item("nav-settings", "Settings", IconName::Settings, Page::Settings, p, cx))
            .child(div().flex_1())
            .child(div().h(px(1.)).bg(rgb(p.border)))
            .child(
                h_flex().id("sidebar-toggle").h(px(42.)).gap_3().px_3()
                    .items_center().rounded_lg().cursor_pointer()
                    .text_color(rgb(p.muted))
                    .hover(move |s| s.bg(rgb(p.hover)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.collapsed = !this.collapsed; cx.notify();
                    }))
                    .child(Icon::new(IconName::PanelLeft))
                    .when(!self.collapsed, |el| el.child(div().text_sm().child("Collapse sidebar")))
            )
            .child(
                h_flex().h(px(50.)).px_2().gap_3().items_center()
                    .child(div().size_8().rounded_full().bg(rgb(p.avatar))
                        .flex().items_center().justify_center().child("W"))
                    .when(!self.collapsed, |el| el.child(
                        v_flex().gap_0()
                            .child(div().text_sm().font_medium().child("Workspace"))
                            .child(div().text_xs().text_color(rgb(p.subtle)).child("Local mode"))
                    ))
            )
    }

    fn header(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().w_full().h(px(62.)).flex_shrink_0()
            .px_7().items_center().justify_between()
            .border_b_1().border_color(rgb(p.border)).bg(rgb(p.surface))
            .child(
                h_flex().gap_2().items_center()
                    .child(div().text_sm().text_color(rgb(p.subtle)).child("Workspace"))
                    .child(div().text_sm().text_color(rgb(p.subtle)).child("/"))
                    .child(div().text_sm().font_semibold().child(self.page.title()))
            )
            .child(
                h_flex().items_center().gap_3()
                    .child(div().px_3().py_1().rounded_full()
                        .bg(rgb(p.status_bg)).text_color(rgb(p.status_text))
                        .text_xs().child("● Local preview"))
                    .child(
                        Button::new("theme-button").small()
                            .label(if self.dark { "Light mode" } else { "Dark mode" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dark = !this.dark; cx.notify();
                            }))
                    )
            )
    }

    fn card(&self, id: &'static str, title: &'static str, description: &'static str,
        icon: IconName, page: Page, p: Palette, cx: &mut Context<Self>
    ) -> impl IntoElement {
        v_flex().id(id).flex_1().min_w_0().h(px(150.)).p_4().gap_3()
            .rounded_xl().border_1().border_color(rgb(p.border))
            .bg(rgb(p.surface)).cursor_pointer()
            .hover(move |s| s.bg(rgb(p.hover)))
            .on_click(cx.listener(move |this, _, _, cx| { this.page = page; cx.notify(); }))
            .child(Icon::new(icon))
            .child(v_flex().gap_1()
                .child(div().text_sm().font_semibold().child(title))
                .child(div().text_xs().text_color(rgb(p.muted)).child(description)))
    }

    fn section_title(&self, title: &'static str, detail: &'static str, p: Palette)
        -> impl IntoElement {
        v_flex().gap_2()
            .child(div().text_2xl().font_semibold().child(title))
            .child(div().text_sm().text_color(rgb(p.muted)).child(detail))
    }

    fn overview(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_7()
            .child(
                v_flex().gap_2().pt_7()
                    .child(div().text_xs().text_color(rgb(p.subtle))
                        .font_semibold().child("YOUR WORKSPACE / START HERE"))
                    .child(div().text_3xl().font_semibold().child("Build something great."))
                    .child(div().text_sm().text_color(rgb(p.muted))
                        .child("A focused, reusable native desktop shell built with Rust and GPUI Kit."))
            )
            .child(
                v_flex().gap_3()
                    .child(div().text_sm().font_semibold().child("Quick start"))
                    .child(
                        h_flex().gap_3()
                            .child(self.card("card-components", "Explore components",
                                "Native inputs, controls and buttons",
                                IconName::GalleryVerticalEnd, Page::Components, p, cx))
                            .child(self.card("card-activity", "View activity",
                                "Local example task history",
                                IconName::Inbox, Page::Activity, p, cx))
                            .child(self.card("card-settings", "Personalize",
                                "Appearance and workspace layout",
                                IconName::Settings, Page::Settings, p, cx))
                    )
            )
            .child(
                v_flex().gap_3()
                    .child(div().text_sm().font_semibold().child("Getting started"))
                    .child(
                        v_flex().gap_4().p_5().rounded_xl()
                            .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
                            .child(
                                h_flex().gap_3().items_center()
                                    .child(Icon::new(IconName::CircleCheck))
                                    .child(div().text_sm().child("GPUI Kit application initialized"))
                            )
                            .child(div().h(px(1.)).bg(rgb(p.border)))
                            .child(
                                h_flex().gap_3().items_center()
                                    .child(Icon::new(IconName::CircleCheck))
                                    .child(div().text_sm().child("Reusable navigation and theme tokens"))
                            )
                            .child(div().h(px(1.)).bg(rgb(p.border)))
                            .child(
                                h_flex().gap_3().items_center()
                                    .child(Icon::new(IconName::CircleCheck))
                                    .child(div().text_sm().child("Windows and macOS release pipeline"))
                            )
                    )
            )
            .child(div().text_xs().text_color(rgb(p.subtle))
                .child("Codex-inspired appearance; this is not an official OpenAI app."))
    }

    fn components(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_6()
            .child(self.section_title("Component gallery",
                "Functional controls from the GPUI Kit component system.", p))
            .child(
                v_flex().gap_5().p_6().rounded_xl()
                    .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
                    .child(div().text_sm().font_semibold().child("Buttons"))
                    .child(
                        h_flex().gap_3().items_center()
                            .child(
                                Button::new("primary-demo").primary().label("Primary action")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clicks += 1; cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("secondary-demo").label("Secondary action")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clicks += 1; cx.notify();
                                    }))
                            )
                            .child(div().text_xs().text_color(rgb(p.muted))
                                .child(SharedString::from(format!("{} clicks", self.clicks))))
                    )
                    .child(div().h(px(1.)).bg(rgb(p.border)))
                    .child(div().text_sm().font_semibold().child("Text input"))
                    .child(div().max_w(px(500.)).child(Input::new(&self.input)))
                    .child(div().h(px(1.)).bg(rgb(p.border)))
                    .child(div().text_sm().font_semibold().child("Toggle"))
                    .child(
                        Switch::new("collapse-switch").label("Collapse sidebar")
                            .checked(self.collapsed)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.collapsed = *checked; cx.notify();
                            }))
                    )
                    .child(div().text_xs().text_color(rgb(p.subtle))
                        .child("Controls update application state and re-render natively."))
            )
    }

    fn activity(&self, p: Palette) -> impl IntoElement {
        v_flex().gap_6()
            .child(self.section_title("Activity",
                "Task examples stored in memory only. No AI backend is connected.", p))
            .child(
                v_flex().rounded_xl()
                    .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
                    .children(self.tasks.iter().rev().enumerate().map(|(i, title)| {
                        h_flex().gap_3().px_5().py_4().items_center()
                            .border_b_1().border_color(rgb(p.border))
                            .child(Icon::new(IconName::CircleCheck))
                            .child(
                                v_flex().flex_1().gap_1()
                                    .child(div().text_sm().font_medium()
                                        .child(SharedString::from(title.clone())))
                                    .child(div().text_xs().text_color(rgb(p.subtle))
                                        .child(SharedString::from(
                                            format!("TASK #{:03} · Local demo", self.tasks.len() - i)
                                        )))
                            )
                            .child(div().text_xs().text_color(rgb(p.status_text)).child("Ready"))
                    }))
            )
    }

    fn settings(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_6()
            .child(self.section_title("Preferences",
                "Customize the workspace for your own product.", p))
            .child(
                v_flex().gap_5().p_6().rounded_xl()
                    .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
                    .child(div().text_sm().font_semibold().child("Appearance"))
                    .child(
                        Switch::new("dark-switch").label("Dark appearance")
                            .checked(self.dark)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.dark = *checked; cx.notify();
                            }))
                    )
                    .child(div().h(px(1.)).bg(rgb(p.border)))
                    .child(div().text_sm().font_semibold().child("Navigation"))
                    .child(
                        Switch::new("sidebar-switch").label("Collapsed sidebar")
                            .checked(self.collapsed)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.collapsed = *checked; cx.notify();
                            }))
                    )
                    .child(div().h(px(1.)).bg(rgb(p.border)))
                    .child(div().text_sm().font_semibold().child("About"))
                    .child(div().text_sm().text_color(rgb(p.muted))
                        .child("GPUI Kit Demo 0.1.0 · Rust · GPUI Kit 0.9.0"))
                    .child(div().text_xs().text_color(rgb(p.subtle))
                        .child("All preferences reset on restart in this starter version."))
            )
    }

    fn composer(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().w_full().gap_2().px_7().pt_3().pb_4()
            .bg(rgb(p.surface)).border_t_1().border_color(rgb(p.border))
            .child(
                h_flex().items_center().gap_3().p_3().rounded_xl()
                    .bg(rgb(p.canvas)).border_1().border_color(rgb(p.border))
                    .child(div().min_w_0().flex_1().child(Input::new(&self.input)))
                    .child(
                        Button::new("add-task").primary().label("Add task")
                            .on_click(cx.listener(|this, _, _, cx| {
                                let entered = this.input.read(cx).value().to_string();
                                if !entered.trim().is_empty() {
                                    this.tasks.push(entered.trim().to_owned());
                                    this.page = Page::Activity;
                                    cx.notify();
                                }
                            }))
                    )
            )
            .child(
                h_flex().items_center().justify_between()
                    .child(div().text_xs().text_color(rgb(p.subtle))
                        .child("Local demo · No AI service or telemetry"))
                    .child(div().text_xs().text_color(rgb(p.subtle))
                        .child("Rust · GPUI Kit 0.9"))
            )
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = Palette::new(self.dark);
        h_flex().size_full().bg(rgb(p.canvas)).text_color(rgb(p.text))
            .child(self.sidebar(p, cx))
            .child(
                v_flex().flex_1().h_full().min_w_0()
                    .child(self.header(p, cx))
                    .child(
                        v_flex().flex_1().min_h_0().items_center().overflow_y_scrollbar()
                            .child(
                                v_flex().w_full().max_w(px(1050.)).p_7()
                                    .child(match self.page {
                                        Page::Overview => self.overview(p, cx).into_any_element(),
                                        Page::Components => self.components(p, cx).into_any_element(),
                                        Page::Activity => self.activity(p).into_any_element(),
                                        Page::Settings => self.settings(p, cx).into_any_element(),
                                    })
                            )
                    )
                    .child(self.composer(p, cx))
            )
    }
}

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1220.), px(800.)), cx)),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        }).expect("Failed to open GPUI Kit Demo");
    });
}

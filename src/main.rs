mod component_gallery;
mod palette;
mod storage;

use gpui_kit::assets::Assets;
use gpui_kit::base::StyledExt;
use gpui_kit::component::Sizable;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::{
    button::*, h_flex, input::{Input, InputState}, switch::Switch, v_flex, Icon, IconName, Theme, ThemeMode,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use component_gallery::GalleryTab;
use palette::Palette;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page { Overview, Components, Activity, Settings }

impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Overview => "Home", Self::Components => "UI Components",
            Self::Activity => "Tasks", Self::Settings => "Preferences",
        }
    }
}

struct Workspace {
    page: Page,
    dark: bool,
    collapsed: bool,
    clicks: usize,
    gallery_tab: GalleryTab,
    gallery_switch: bool,
    gallery_checkbox: bool,
    gallery_selection: usize,
    gallery_preview_text: String,
    tasks: Vec<String>,
    input: Entity<InputState>,
    playground_input: Entity<InputState>,
    save_error: Option<String>,
}

impl Workspace {
    fn new(window: &mut Window, cx: &mut Context<Self>, settings: storage::Settings, save_error: Option<String>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx)
            .placeholder("Add a task to your workspace"));
        let playground_input = cx.new(|cx| InputState::new(window, cx)
            .placeholder("Type to preview the native input"));
        Self {
            page: Page::Overview, dark: settings.dark, collapsed: settings.collapsed, clicks: 0,
            gallery_tab: GalleryTab::Controls, gallery_switch: false, gallery_checkbox: false,
            gallery_selection: 0, gallery_preview_text: String::new(),
            tasks: settings.tasks, input, playground_input, save_error,
        }
    }

    fn persist(&mut self) {
        let settings = storage::Settings {
            dark: self.dark, collapsed: self.collapsed, tasks: self.tasks.clone(),
        };
        self.save_error = storage::save(&settings).err().map(|err| err.to_string());
    }

    fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.dark = dark;
        Theme::change(if dark { ThemeMode::Dark } else { ThemeMode::Light }, None, cx);
        self.persist();
        cx.notify();
    }

    fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.collapsed = collapsed;
        self.persist();
        cx.notify();
    }

    fn menu_item(
        &self, id: &'static str, label: &'static str, icon: IconName,
        page: Page, p: Palette, cx: &mut Context<Self>
    ) -> impl IntoElement {
        let active = self.page == page;
        h_flex().id(id).w_full().h(px(36.)).px_3().gap_3()
            .items_center().cursor_pointer().rounded_md()
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
        v_flex().h_full().w(px(if self.collapsed { 64. } else { 224. }))
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
                            .child(div().text_sm().font_semibold().child("Workbench"))
                            .child(div().text_xs().text_color(rgb(p.subtle)).child("GPUI KIT / NATIVE"))
                    ))
            )
            .child(div().h(px(14.)))
            .when(!self.collapsed, |el| el.child(
                div().px_3().pb_2().text_xs().text_color(rgb(p.subtle))
                    .child("NAVIGATION")
            ))
            .child(self.menu_item("nav-overview", "Home", IconName::LayoutDashboard, Page::Overview, p, cx))
            .child(self.menu_item("nav-components", "UI Components", IconName::GalleryVerticalEnd, Page::Components, p, cx))
            .child(self.menu_item("nav-activity", "Tasks", IconName::Inbox, Page::Activity, p, cx))
            .child(div().h(px(18.)))
            .when(!self.collapsed, |el| el.child(
                div().px_3().pb_2().text_xs().text_color(rgb(p.subtle))
                    .child("SYSTEM")
            ))
            .child(self.menu_item("nav-settings", "Preferences", IconName::Settings, Page::Settings, p, cx))
            .child(div().flex_1())
            .child(div().h(px(1.)).bg(rgb(p.border)))
            .child(
                h_flex().id("sidebar-toggle").h(px(42.)).gap_3().px_3()
                    .items_center().rounded_lg().cursor_pointer()
                    .text_color(rgb(p.muted))
                    .hover(move |s| s.bg(rgb(p.hover)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.set_collapsed(!this.collapsed, cx);
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
                            .child(div().text_sm().font_medium().child("On this device"))
                            .child(div().text_xs().text_color(rgb(p.subtle)).child("No account needed"))
                    ))
            )
    }

    fn header(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().w_full().h(px(58.)).flex_shrink_0()
            .px_7().items_center().justify_between()
            .border_b_1().border_color(rgb(p.border)).bg(rgb(p.surface))
            .child(
                h_flex().gap_2().items_center()
                    .child(div().text_sm().text_color(rgb(p.subtle)).child("Workbench"))
                    .child(div().text_sm().text_color(rgb(p.subtle)).child("/"))
                    .child(div().text_sm().font_semibold().child(self.page.title()))
            )
            .child(
                h_flex().items_center().gap_4()
                    .child(div().text_xs().text_color(rgb(p.muted)).child("LOCAL"))
                    .child(
                        Button::new("theme-button").small()
                            .label(if self.dark { "Light appearance" } else { "Dark appearance" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_dark(!this.dark, cx);
                            }))
                    )
            )
    }

    fn destination_row(
        &self, id: &'static str, title: &'static str, description: &'static str,
        icon: IconName, page: Page, p: Palette, cx: &mut Context<Self>
    ) -> impl IntoElement {
        h_flex().id(id).w_full().min_w_0().gap_4().px_5().py_4()
            .items_center().rounded_lg().cursor_pointer()
            .bg(rgb(p.surface)).border_1().border_color(rgb(p.border))
            .hover(move |s| s.bg(rgb(p.hover)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.page = page;
                cx.notify();
            }))
            .child(div().size_9().rounded_md().bg(rgb(p.accent_soft))
                .text_color(rgb(p.accent_ink)).flex().items_center()
                .justify_center().child(Icon::new(icon)))
            .child(v_flex().flex_1().min_w_0().gap_1()
                .child(div().text_sm().font_medium().child(title))
                .child(div().text_xs().text_color(rgb(p.muted)).child(description)))
            .child(div().text_xs().text_color(rgb(p.subtle)).child("Open"))
    }

    fn section_title(&self, title: &'static str, detail: &'static str, p: Palette)
        -> impl IntoElement {
        v_flex().gap_2()
            .child(div().text_2xl().font_semibold().child(title))
            .child(div().text_sm().text_color(rgb(p.muted)).child(detail))
    }

    fn overview(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_6()
            .child(
                v_flex().gap_2().pt_4()
                    .child(div().text_xs().font_semibold()
                        .text_color(rgb(p.accent_ink)).child("WORKBENCH / OVERVIEW"))
                    .child(div().text_3xl().font_semibold().child("Workspace overview"))
                    .child(div().text_sm().text_color(rgb(p.muted))
                        .child("Local tasks, native controls, and appearance settings."))
            )
            .child(
                v_flex().gap_4().p_6().rounded_lg()
                    .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
                    .child(
                        h_flex().gap_3().items_center()
                            .child(div().text_xs().font_semibold().text_color(rgb(p.subtle))
                                .child("YOUR TASKS"))
                            .child(div().flex_1())
                            .child(div().text_xs().text_color(rgb(p.muted))
                                .child("Stored on this device"))
                    )
                    .child(div().text_3xl().font_semibold()
                        .child(SharedString::from(format!("{}", self.tasks.len()))))
                    .child(div().text_sm().text_color(rgb(p.muted))
                        .child(if self.tasks.len() == 1 { "Task in your list" } else { "Tasks in your list" }))
                    .child(
                        h_flex().items_center()
                            .child(Button::new("overview-activity").primary().label("Open task list")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.page = Page::Activity;
                                    cx.notify();
                                })))
                    )
            )
            .child(
                v_flex().gap_3()
                    .child(div().text_sm().font_semibold().child("Explore the workspace"))
                    .child(self.destination_row("overview-components", "UI component gallery",
                        "Explore native controls, interface patterns, and design tokens",
                        IconName::GalleryVerticalEnd, Page::Components, p, cx))
                    .child(self.destination_row("overview-preferences", "Appearance & layout",
                        "Switch themes and adjust the navigation",
                        IconName::Settings, Page::Settings, p, cx))
            )
            .child(div().text_xs().text_color(rgb(p.subtle))
                .child("A native desktop reference project. Everything runs locally."))
    }

    fn activity(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_5()
            .child(self.section_title("Tasks",
                "A small, persistent task list for trying native interactions.", p))
            .child(self.task_editor(p, cx))
            .child(div().text_sm().font_semibold()
                .child(SharedString::from(format!("Saved tasks · {}", self.tasks.len()))))
            .child(
                v_flex().rounded_lg()
                    .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
                    .when(self.tasks.is_empty(), |el| el.child(
                        v_flex().gap_2().p_8().items_center()
                            .child(Icon::new(IconName::Inbox))
                            .child(div().text_sm().font_medium().child("Nothing here yet"))
                            .child(div().text_xs().text_color(rgb(p.muted))
                                .child("Create a task above to populate this list."))
                    ))
                    .children(self.tasks.iter().rev().enumerate().map(|(i, title)| {
                        let index = self.tasks.len() - 1 - i;
                        h_flex().gap_3().px_5().py_4().items_center()
                            .border_b_1().border_color(rgb(p.border))
                            .child(Icon::new(IconName::CircleCheck))
                            .child(
                                v_flex().flex_1().gap_1().min_w_0()
                                    .child(div().text_sm().font_medium()
                                        .child(SharedString::from(title.clone())))
                                    .child(div().text_xs().text_color(rgb(p.subtle))
                                        .child(SharedString::from(
                                            format!("LOCAL TASK / {:03}", index + 1)
                                        )))
                            )
                            .child(
                                Button::new(SharedString::from(format!("remove-task-{}", index)))
                                    .small().ghost().label("Remove")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if index < this.tasks.len() {
                                            this.tasks.remove(index);
                                            this.persist();
                                            cx.notify();
                                        }
                                    }))
                            )
                    }))
            )
    }

    fn settings(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_6()
            .child(self.section_title("Appearance & layout",
                "A small set of durable, device-specific preferences.", p))
            .child(
                v_flex().gap_5().p_6().rounded_xl()
                    .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
                    .child(div().text_sm().font_semibold().child("Appearance"))
                    .child(
                        Switch::new("dark-switch").label("Dark appearance")
                            .checked(self.dark)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.set_dark(*checked, cx);
                            }))
                    )
                    .child(div().h(px(1.)).bg(rgb(p.border)))
                    .child(div().text_sm().font_semibold().child("Navigation"))
                    .child(
                        Switch::new("sidebar-switch").label("Collapsed sidebar")
                            .checked(self.collapsed)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.set_collapsed(*checked, cx);
                            }))
                    )
                    .child(div().h(px(1.)).bg(rgb(p.border)))
                    .child(div().text_sm().font_semibold().child("About"))
                    .child(div().text_sm().text_color(rgb(p.muted))
                        .child("GPUI Kit Demo 0.3.0 · Rust · GPUI Kit 0.9.0"))
                    .child(div().text_xs().text_color(rgb(p.subtle))
                        .child("Appearance, sidebar and tasks are saved locally between sessions."))
            )
    }

    fn task_editor(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().w_full().gap_3().p_5().rounded_lg()
            .bg(rgb(p.surface)).border_1().border_color(rgb(p.border))
            .child(div().text_sm().font_medium().child("New task"))
            .child(
                h_flex().items_center().gap_3()
                    .child(div().min_w_0().flex_1().child(Input::new(&self.input)))
                    .child(
                        Button::new("add-task").primary().label("Add task")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let entered = this.input.read(cx).value().to_string();
                                if !entered.trim().is_empty() {
                                    if this.tasks.len() < storage::MAX_TASKS {
                                        let title: String = entered.trim().chars().take(storage::MAX_TASK_CHARS).collect();
                                        this.tasks.push(title);
                                        this.input.update(cx, |input, cx| input.set_value("", window, cx));
                                        this.page = Page::Activity;
                                        this.persist();
                                        cx.notify();
                                    }
                                }
                            }))
                    )
            )
            .child(div().text_xs().text_color(rgb(p.subtle))
                .child(SharedString::from(format!(
                    "Saved automatically · {} / {} tasks", self.tasks.len(), storage::MAX_TASKS
                ))))
    }

    fn status_bar(&self, p: Palette) -> impl IntoElement {
        h_flex().w_full().h(px(34.)).flex_shrink_0().px_7()
            .items_center().justify_between()
            .border_t_1().border_color(rgb(p.border)).bg(rgb(p.surface))
            .child(div().text_xs().text_color(rgb(p.subtle))
                .child("GPUI KIT · NATIVE DESKTOP"))
            .child(div().text_xs()
                .text_color(rgb(if self.save_error.is_some() { p.accent_ink } else { p.muted }))
                .child(SharedString::from(
                    self.save_error.as_ref().map(|err| format!("Save error: {err}"))
                        .unwrap_or_else(|| "Local storage ready".to_string())
                )))
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
                        v_flex().flex_1().min_h_0().items_center().id("workspace-scroll").overflow_y_scrollbar()
                            .child(
                                v_flex().w_full().max_w(px(1050.)).p_7()
                                    .child(match self.page {
                                        Page::Overview => self.overview(p, cx).into_any_element(),
                                        Page::Components => self.component_gallery(p, cx).into_any_element(),
                                        Page::Activity => self.activity(p, cx).into_any_element(),
                                        Page::Settings => self.settings(p, cx).into_any_element(),
                                    })
                            )
                    )
                    .child(self.status_bar(p))
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
                (storage::Settings::default(), Some(format!("Could not restore settings: {err}")))
            }
        };
        Theme::change(if settings.dark { ThemeMode::Dark } else { ThemeMode::Light }, None, cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1220.), px(800.)), cx)),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, move |window, cx| {
            cx.new(|cx| Workspace::new(window, cx, settings, save_error))
        }).expect("Failed to open GPUI Kit Demo");
    });
}

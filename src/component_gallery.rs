//! Interactive, native GPUI Kit component previews.
//! Gallery-only state is intentionally kept out of persisted Settings.

use crate::{Page, Workspace, palette::Palette};
use gpui_kit::base::StyledExt;
use gpui_kit::component::Disableable;
use gpui_kit::component::Sizable;
use gpui_kit::component::{
    Icon, IconName, button::*, checkbox::Checkbox, h_flex, input::Input,
    switch::Switch, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GalleryTab {
    Controls,
    Patterns,
    Foundations,
}

impl GalleryTab {
    fn label(self) -> &'static str {
        match self {
            Self::Controls => "Controls",
            Self::Patterns => "Patterns",
            Self::Foundations => "Foundations",
        }
    }
}

impl Workspace {
    pub(super) fn component_gallery(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().w_full().gap_5()
            .child(
                h_flex().w_full().items_center().justify_between().gap_4()
                    .child(
                        v_flex().gap_2().pt_2()
                            .child(div().text_xs().font_semibold()
                                .text_color(rgb(p.accent_ink)).child("NATIVE UI / REFERENCE"))
                            .child(div().text_3xl().font_semibold().child("UI components"))
                            .child(div().text_sm().text_color(rgb(p.muted))
                                .child("Reusable controls and patterns from this workbench."))
                    )
                    .child(
                        Button::new("gallery-appearance").small()
                            .label(if self.dark { "Light mode" } else { "Dark mode" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_dark(!this.dark, cx);
                            }))
                    )
            )
            .child(
                h_flex().w_full().gap_2().p_1().rounded_lg()
                    .bg(rgb(p.sidebar))
                    .child(self.gallery_tab_button("gallery-tab-controls", GalleryTab::Controls, cx))
                    .child(self.gallery_tab_button("gallery-tab-patterns", GalleryTab::Patterns, cx))
                    .child(self.gallery_tab_button("gallery-tab-foundations", GalleryTab::Foundations, cx))
            )
            .child(match self.gallery_tab {
                GalleryTab::Controls => self.gallery_controls(p, cx).into_any_element(),
                GalleryTab::Patterns => self.gallery_patterns(p, cx).into_any_element(),
                GalleryTab::Foundations => self.gallery_foundations(p).into_any_element(),
            })
            .child(div().pt_1().text_xs().text_color(rgb(p.subtle))
                .child("These previews use the same GPUI Kit components as the rest of the app."))
    }

    fn gallery_tab_button(
        &self, id: &'static str, tab: GalleryTab, cx: &mut Context<Self>
    ) -> impl IntoElement {
        Button::new(id)
            .with_variant(if self.gallery_tab == tab {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Ghost
            })
            .label(tab.label())
            .on_click(cx.listener(move |this, _, _, cx| {
                this.gallery_tab = tab;
                cx.notify();
            }))
    }

    fn gallery_panel(
        &self, heading: &'static str, detail: &'static str, p: Palette,
        preview: impl IntoElement,
    ) -> impl IntoElement {
        v_flex().w_full().gap_4().p_5().rounded_lg()
            .border_1().border_color(rgb(p.border)).bg(rgb(p.surface))
            .child(
                v_flex().gap_1()
                    .child(div().text_sm().font_semibold().child(heading))
                    .child(div().text_xs().text_color(rgb(p.muted)).child(detail))
            )
            .child(div().h(px(1.)).bg(rgb(p.border)))
            .child(preview)
    }

    fn gallery_controls(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().w_full().gap_4()
            .child(self.gallery_panel(
                "Buttons", "GPUI Kit · primary, secondary, ghost and disabled", p,
                v_flex().gap_3()
                    .child(
                        h_flex().gap_3().items_center()
                            .child(Button::new("gallery-primary").primary()
                                .label("Primary")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.clicks += 1;
                                    cx.notify();
                                })))
                            .child(Button::new("gallery-secondary").secondary()
                                .label("Secondary")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.clicks += 1;
                                    cx.notify();
                                })))
                            .child(Button::new("gallery-ghost").ghost()
                                .label("Ghost")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.clicks += 1;
                                    cx.notify();
                                })))
                    )
                    .child(
                        h_flex().items_center().gap_3()
                            .child(Button::new("gallery-small").small().label("Small action")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.clicks += 1;
                                    cx.notify();
                                })))
                            .child(Button::new("gallery-disabled").label("Disabled").disabled(true))
                            .child(Button::new("gallery-reset").ghost().small()
                                .label("Reset")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.clicks = 0;
                                    cx.notify();
                                })))
                    )
                    .child(div().text_xs().text_color(rgb(p.muted))
                        .child(SharedString::from(format!("Action activations: {}", self.clicks))))
            ))
            .child(self.gallery_panel(
                "Text input", "GPUI Kit · native editable field", p,
                v_flex().gap_3()
                    .child(div().max_w(px(560.)).child(Input::new(&self.playground_input)))
                    .child(
                        h_flex().items_center().gap_3()
                            .child(Button::new("gallery-read-input").secondary().small()
                                .label("Use current text")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.gallery_preview_text =
                                        this.playground_input.read(cx).value().to_string();
                                    cx.notify();
                                })))
                            .child(div().text_xs().text_color(rgb(p.muted))
                                .child(SharedString::from(if self.gallery_preview_text.is_empty() {
                                    "No preview text selected".to_string()
                                } else {
                                    format!("Selected: {}", self.gallery_preview_text)
                                })))
                    )
            ))
            .child(self.gallery_panel(
                "Selection controls", "GPUI Kit · switch and checkbox", p,
                v_flex().gap_4()
                    .child(
                        Switch::new("gallery-preview-switch").label("Enable preview option")
                            .checked(self.gallery_switch)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.gallery_switch = *checked;
                                cx.notify();
                            }))
                    )
                    .child(
                        Checkbox::new("gallery-preview-checkbox").label("Include this item")
                            .checked(self.gallery_checkbox)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.gallery_checkbox = *checked;
                                cx.notify();
                            }))
                    )
                    .child(div().text_xs().text_color(rgb(p.muted))
                        .child(SharedString::from(format!(
                            "Switch: {} · Checkbox: {}",
                            if self.gallery_switch { "on" } else { "off" },
                            if self.gallery_checkbox { "checked" } else { "unchecked" },
                        ))))
            ))
    }

    fn gallery_pattern_button(
        &self, id: &'static str, label: &'static str, index: usize, cx: &mut Context<Self>
    ) -> impl IntoElement {
        Button::new(id)
            .with_variant(if self.gallery_selection == index {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Ghost
            })
            .label(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.gallery_selection = index;
                cx.notify();
            }))
    }

    fn gallery_patterns(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let selection_label = match self.gallery_selection {
            0 => "Overview",
            1 => "Components",
            _ => "Tasks",
        };
        v_flex().w_full().gap_4()
            .child(self.gallery_panel(
                "Navigation", "A local selection preview, independent from the sidebar", p,
                v_flex().gap_3()
                    .child(
                        h_flex().gap_2().items_center()
                            .child(self.gallery_pattern_button(
                                "gallery-nav-overview", "Overview", 0, cx))
                            .child(self.gallery_pattern_button(
                                "gallery-nav-components", "Components", 1, cx))
                            .child(self.gallery_pattern_button(
                                "gallery-nav-tasks", "Tasks", 2, cx))
                    )
                    .child(div().p_4().rounded_md().bg(rgb(p.canvas))
                        .child(div().text_sm().text_color(rgb(p.muted))
                            .child(SharedString::from(format!(
                                "Selected preview: {selection_label}"
                            )))))
            ))
            .child(self.gallery_panel(
                "Workspace navigation", "These buttons operate the actual application", p,
                v_flex().gap_3()
                    .child(
                        h_flex().gap_3().items_center()
                            .child(
                                Button::new("gallery-go-tasks").primary().label("Open tasks")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.page = Page::Activity;
                                        cx.notify();
                                    }))
                            )
                            .child(
                                Button::new("gallery-collapse-sidebar").secondary()
                                    .label(if self.collapsed {
                                        "Expand sidebar"
                                    } else {
                                        "Collapse sidebar"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.set_collapsed(!this.collapsed, cx);
                                    }))
                            )
                    )
                    .child(div().text_xs().text_color(rgb(p.muted))
                        .child(if self.collapsed {
                            "Sidebar is collapsed; this preference is saved locally."
                        } else {
                            "Sidebar is expanded; this preference is saved locally."
                        }))
            ))
            .child(self.gallery_panel(
                "Status and empty states", "Real saved-state feedback and live task data", p,
                v_flex().gap_3()
                    .child(
                        h_flex().gap_3().p_4().rounded_md()
                            .bg(rgb(p.status_bg)).text_color(rgb(p.status_text))
                            .child(
                                div().text_sm().child(SharedString::from(
                                    self.save_error.as_ref()
                                        .map(|err| format!("Save error: {err}"))
                                        .unwrap_or_else(|| "Local storage ready".to_string())
                                ))
                            )
                    )
                    .child(
                        h_flex().gap_3().items_center().p_4().rounded_md()
                            .border_1().border_color(rgb(p.border))
                            .child(Icon::new(IconName::Inbox))
                            .child(
                                v_flex().flex_1().gap_1().min_w_0()
                                    .child(div().text_sm().font_medium()
                                        .child(SharedString::from(if let Some(task) = self.tasks.last() {
                                            task.clone()
                                        } else {
                                            "No tasks yet".to_string()
                                        })))
                                    .child(div().text_xs().text_color(rgb(p.muted))
                                        .child(SharedString::from(if self.tasks.is_empty() {
                                            "Open Tasks to create your first item".to_string()
                                        } else {
                                            format!("Latest local task · {} total", self.tasks.len())
                                        })))
                            )
                    )
                    .child(
                        Button::new("gallery-open-task-list").small().ghost().label("View task list")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.page = Page::Activity;
                                cx.notify();
                            }))
                    )
            ))
    }

    fn gallery_swatch(&self, name: &'static str, value: u32, p: Palette) -> impl IntoElement {
        v_flex().gap_2().w(px(112.))
            .child(div().w_full().h(px(52.)).rounded_md()
                .bg(rgb(value)).border_1().border_color(rgb(p.border)))
            .child(div().text_xs().font_medium().child(name))
            .child(div().text_xs().text_color(rgb(p.muted))
                .child(SharedString::from(format!("#{value:06X}"))))
    }

    fn gallery_foundations(&self, p: Palette) -> impl IntoElement {
        v_flex().w_full().gap_4()
            .child(self.gallery_panel(
                "Typography", "The workbench text hierarchy in the current theme", p,
                v_flex().gap_3()
                    .child(div().text_2xl().font_semibold().child("Page heading"))
                    .child(div().text_sm().font_semibold().child("Section heading"))
                    .child(div().text_sm().child("Body text for labels and useful descriptions."))
                    .child(div().text_xs().text_color(rgb(p.muted))
                        .child("Supporting text / muted"))
                    .child(div().text_xs().text_color(rgb(p.subtle))
                        .child("Caption / subtle"))
            ))
            .child(self.gallery_panel(
                "Semantic colors", "Values come from src/palette.rs, not showcase-only colors", p,
                v_flex().gap_4()
                    .child(h_flex().gap_3()
                        .child(self.gallery_swatch("Canvas", p.canvas, p))
                        .child(self.gallery_swatch("Surface", p.surface, p))
                        .child(self.gallery_swatch("Sidebar", p.sidebar, p))
                        .child(self.gallery_swatch("Border", p.border, p)))
                    .child(h_flex().gap_3()
                        .child(self.gallery_swatch("Selected", p.selected, p))
                        .child(self.gallery_swatch("Accent soft", p.accent_soft, p))
                        .child(self.gallery_swatch("Accent ink", p.accent_ink, p))
                        .child(self.gallery_swatch("Status", p.status_bg, p)))
            ))
            .child(self.gallery_panel(
                "Surface hierarchy", "Spacing and quiet borders keep content legible", p,
                v_flex().gap_3().p_4().rounded_md().bg(rgb(p.canvas))
                    .child(div().text_xs().text_color(rgb(p.muted)).child("Canvas background"))
                    .child(
                        v_flex().gap_2().p_4().rounded_lg().bg(rgb(p.surface))
                            .border_1().border_color(rgb(p.border))
                            .child(div().text_sm().font_semibold().child("Surface"))
                            .child(div().text_xs().text_color(rgb(p.muted))
                                .child("Reusable panel with a subtle border"))
                    )
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::GalleryTab;

    #[test]
    fn categories_have_distinct_labels() {
        let labels = [
            GalleryTab::Controls.label(),
            GalleryTab::Patterns.label(),
            GalleryTab::Foundations.label(),
        ];
        assert_eq!(labels.len(), 3);
        assert_ne!(labels[0], labels[1]);
        assert_ne!(labels[1], labels[2]);
        assert_ne!(labels[0], labels[2]);
    }
}

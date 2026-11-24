//! Small reusable presentational builders shared across workbench pages.
//! Every helper takes its colors from [`crate::palette::Palette`] so screens
//! never hard-code colors locally.
//!
//! The set below is the app's own design kit:
//! - [`card`] outer interactive container
//! - [`stage`] tinted demo area inside a card
//! - [`section_head`] numbered card header
//! - [`field_label`] small caps group label
//! - [`caption`] quiet supporting copy
//! - [`pill`] rounded meta chip
//! - [`setting_row`] description row with a trailing control slot
//! - [`swatch`] semantic color preview tile
//! - [`divider`] 1px separator

use crate::palette::Palette;
use gpui_kit::StatefulInteractiveElement as _;
use gpui_kit::base::StyledExt as _;
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{Icon, IconName, h_flex, v_flex};
use gpui_kit::{
    Div, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb,
};

/// Outer container that hosts real content: Surface fill, hairline border,
/// medium radius. The fill stays slightly translucent so the frosted window
/// backdrop extends beneath the whole stack; content remains readable because
/// the canvas and surface layers composite to near-opaque.
pub fn card(p: Palette) -> Div {
    div()
        .bg(rgba(p.surface, p.alpha_surface))
        .border_1()
        .border_color(rgb(p.border))
        .rounded_xl()
}

/// Quiet "stage" area inside a card where live controls are demonstrated.
/// Uses the Canvas tone so demos read as a sub-layer of the Surface.
pub fn stage(p: Palette) -> Div {
    div()
        .bg(rgb(p.canvas))
        .border_1()
        .border_color(rgb(p.border))
        .rounded_lg()
}

/// Numbered header used by gallery cards: clay chip with the section number
/// next to its title and one-line description.
pub fn section_head(no: &'static str, title: &'static str, desc: &'static str, p: Palette) -> Div {
    h_flex()
        .gap_3()
        .items_center()
        .child(
            div()
                .flex_shrink_0()
                .size_7()
                .rounded_md()
                .bg(rgb(p.accent_soft))
                .text_color(rgb(p.accent_ink))
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .font_semibold()
                .child(no),
        )
        .child(
            v_flex()
                .min_w_0()
                .child(div().text_sm().font_semibold().child(title))
                .child(div().text_xs().text_color(rgb(p.muted)).child(desc)),
        )
}

/// Small caps group label, e.g. `VARIANTS`, `SIZES`.
pub fn field_label(text: &'static str, p: Palette) -> Div {
    div()
        .text_xs()
        .font_semibold()
        .text_color(rgb(p.subtle))
        .child(text)
}

/// Quiet supporting copy under a demo.
pub fn caption(text: impl Into<SharedString>, p: Palette) -> Div {
    let text: SharedString = text.into();
    div().text_xs().text_color(rgb(p.subtle)).child(text)
}

/// Rounded meta chip used in page headers.
pub fn pill(text: &'static str, p: Palette) -> Div {
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
        .child(text)
}

/// Full-width 1px separator.
pub fn divider(p: Palette) -> Div {
    div().w_full().h(px(1.)).bg(rgb(p.border)).flex_shrink_0()
}

/// Semantic color with alpha, used by the frosted-window surfaces. Alpha 1
/// renders the same role fully opaque.
pub fn rgba(hex: u32, alpha: f32) -> gpui_kit::Hsla {
    let color: gpui_kit::Hsla = rgb(hex).into();
    color.opacity(alpha)
}

/// A settings entry: soft icon tile on the left, label + description, control
/// on the right, with a quiet hover sweep for affordance.
pub fn setting_row(
    id: &'static str,
    icon: IconName,
    title: &'static str,
    desc: &'static str,
    p: Palette,
    control: impl IntoElement,
) -> impl IntoElement {
    div()
        .id(id)
        .w_full()
        .gap_4()
        .items_center()
        .px_2()
        .py_1()
        .rounded_lg()
        .cursor_default()
        .hover(move |style| style.bg(rgb(p.hover)))
        .child(
            div()
                .flex_shrink_0()
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
                .child(div().text_xs().text_color(rgb(p.muted)).child(desc)),
        )
        .child(control.into_any_element())
        .test_support()
        .aria_label(title)
}

/// Semantic color preview: swatch, role name and hex value of the current mode.
pub fn swatch(role: &'static str, hex: u32, p: Palette) -> Div {
    v_flex()
        .gap_2()
        .w(px(96.))
        .child(
            div()
                .h(px(44.))
                .rounded_md()
                .border_1()
                .border_color(rgb(p.border))
                .bg(rgb(hex)),
        )
        .child(
            v_flex()
                .child(div().text_xs().font_medium().child(role))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(p.subtle))
                        .child(SharedString::from(format!("#{hex:06X}"))),
                ),
        )
}

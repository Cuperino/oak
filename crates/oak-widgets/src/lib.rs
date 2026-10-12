//! Oak's custom widget modules, lifted from the oak-gpui fork onto
//! gpui-kit: effect stack, node graph, timeline (the fork's `gpui`
//! additions) and the Oak widget library (the fork's `gpui_widgets`).
//!
//! Everything here builds on public gpui-kit APIs only. The fork's `dock`
//! module is deliberately NOT lifted: gpui-kit's built-in
//! `gpui_kit::component::dock` replaces it.

pub mod audio_meter;
pub mod curve_editor;
pub mod dialog;
pub mod effect_stack;
pub mod i18n;
pub mod icons;
pub mod keyable;
pub mod menu;
pub mod node_graph;
pub mod project_explorer;
pub mod scopes;
pub mod slider;
pub mod theme;
pub mod timeline;
pub mod tooltip;
pub mod value;
pub mod viewer;

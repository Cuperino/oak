// Oak Video Editor - Non-Linear Video Editor
// Copyright (C) 2026 Oak Team
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! `oak-app-new` — the Oak editor shell rebuilt on **gpui-kit** (the
//! registry crates `gpui-pre` + `gpui-base` + `gpui-component`) and the
//! **oak-engine-rust-sdk** C-ABI client. Nothing here links the in-process
//! oak-* module crates or the oak-gpui fork: the UI talks to the engine
//! exclusively through the SDK, and every widget is a gpui-kit component
//! (dock layout, input, list primitives).
//!
//! # Layout
//!
//! * [`app`] — the window shell: theme, dock area, panel wiring.
//! * [`engine`] — the engine seam: the [`engine::AppEngine`] trait with the
//!   SDK-backed [`engine::SdkEngine`] and the demo [`engine::MockEngine`].
//! * [`i18n`] — YAML language packs (runtime discovered, hot-overridable).
//! * [`panels`] — the dockable panels; each implements the gpui-kit dock
//!   [`gpui_kit::component::dock::Panel`] pattern.

pub mod app;
pub mod engine;
pub mod i18n;
pub mod panels;

/// Runs the application (see [`app::run`]).
pub fn run() {
	app::run();
}

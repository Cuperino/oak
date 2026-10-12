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

//! The dockable panels of the main window.
//!
//! Every panel implements the gpui-kit dock pattern —
//! [`gpui_kit::component::dock::BasePanel`] for identity and persistence,
//! [`gpui_kit::component::dock::Panel`] for the tab presentation — holds
//! the engine entity, and never talks to the SDK directly (the
//! [`crate::engine::AppEngine`] seam owns that).
//!
//! Panels whose engine APIs the SDK does not expose yet are stubs (see
//! [`stub_panel!`]); they keep their place in the layout and gain their
//! real UI as the seam grows.

pub mod effect_library;
pub mod history;
pub mod inspector;
pub mod multicam;
pub mod node_editor;
pub mod program_viewer;
pub mod project_explorer;
pub mod source_viewer;
pub mod timeline;

/// Generates a placeholder dock panel: identity + focus + a centered
/// "coming soon" body. Used by the panels whose engine surface the SDK
/// does not expose yet; a stub keeps the panel's name and place in the
/// dock layout stable while its real UI is written.
macro_rules! stub_panel {
	($ty:ident, $panel_name:literal, $title_key:literal) => {
		/// A placeholder panel (real UI pending the engine SDK surface).
		pub struct $ty<E: crate::engine::AppEngine> {
			#[allow(dead_code)]
			engine: gpui_kit::Entity<E>,
			focus_handle: gpui_kit::FocusHandle,
		}

		impl<E: crate::engine::AppEngine> $ty<E> {
			pub fn new(
				engine: gpui_kit::Entity<E>,
				_window: &mut gpui_kit::Window,
				cx: &mut gpui_kit::Context<Self>,
			) -> Self {
				Self {
					engine,
					focus_handle: cx.focus_handle(),
				}
			}
		}

		impl<E: crate::engine::AppEngine>
			gpui_kit::EventEmitter<gpui_kit::component::dock::PanelEvent> for $ty<E>
		{
		}

		impl<E: crate::engine::AppEngine> gpui_kit::Focusable for $ty<E> {
			fn focus_handle(&self, _cx: &gpui_kit::App) -> gpui_kit::FocusHandle {
				self.focus_handle.clone()
			}
		}

		impl<E: crate::engine::AppEngine> gpui_kit::component::dock::BasePanel for $ty<E> {
			fn panel_name(&self) -> &'static str {
				$panel_name
			}
		}

		impl<E: crate::engine::AppEngine> gpui_kit::component::dock::Panel for $ty<E> {
			fn title(
				&mut self,
				_window: &mut gpui_kit::Window,
				_cx: &mut gpui_kit::Context<Self>,
			) -> impl gpui_kit::IntoElement {
				crate::i18n::tr($title_key)
			}
		}

		impl<E: crate::engine::AppEngine> gpui_kit::Render for $ty<E> {
			fn render(
				&mut self,
				_window: &mut gpui_kit::Window,
				cx: &mut gpui_kit::Context<Self>,
			) -> impl gpui_kit::IntoElement {
				use gpui_kit::colors::DefaultColors as _;
				use gpui_kit::{InteractiveElement as _, ParentElement as _, Styled as _};

				let colors = cx.default_colors().clone();
				gpui_kit::div()
					.id($panel_name)
					.size_full()
					.flex()
					.items_center()
					.justify_center()
					.text_sm()
					.text_color(colors.disabled)
					.child(crate::i18n::tr($title_key))
			}
		}
	};
}

pub(crate) use stub_panel;

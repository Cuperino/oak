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

//! The window shell: gpui-kit bootstrap, theme, and the dock layout that
//! hosts the panels.
//!
//! The dock is gpui-kit's own ([`DockSkin`] over
//! [`gpui_kit::component::dock::DockArea`]): panels are added by placement
//! (project explorer left, viewers + node editor center, inspector +
//! effect library right, timeline + history bottom), and the skin renders
//! the tab bars and drop indicators.

use std::rc::Rc;

use gpui_kit::component::dock::{DockArea, DockPlacement, DockSkin};
use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{
	App, AppContext as _, Bounds, Context, Entity, IntoElement, ParentElement as _, Render,
	Styled as _, TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, size,
};

use crate::engine::{AppEngine, MockEngine, SdkEngine};
use crate::panels::effect_library::EffectLibraryPanel;
use crate::panels::history::HistoryPanel;
use crate::panels::inspector::InspectorPanel;
use crate::panels::multicam::MulticamPanel;
use crate::panels::node_editor::NodeEditorPanel;
use crate::panels::program_viewer::ProgramViewerPanel;
use crate::panels::project_explorer::ProjectExplorerPanel;
use crate::panels::source_viewer::SourceViewerPanel;
use crate::panels::timeline::TimelinePanel;
use crate::i18n;

/// The main window: the dock area hosting every panel.
pub struct AppShell<E: AppEngine> {
	#[allow(dead_code)]
	engine: Entity<E>,
	dock: Entity<DockArea>,
	/// Kept alive for the window's lifetime; the skin's settings are
	/// changed through it.
	#[allow(dead_code)]
	skin: Rc<DockSkin>,
}

impl<E: AppEngine> AppShell<E> {
	/// Builds the shell over `engine` and docks every panel.
	pub fn new(engine: Entity<E>, window: &mut Window, cx: &mut Context<Self>) -> Self {
		let (dock, skin) = DockSkin::dock_area("oak-main", Some(1), window, cx);
		dock.update(cx, |dock, cx| {
			dock.add_panel(
				cx.new(|cx| ProjectExplorerPanel::new(engine.clone(), window, cx)),
				DockPlacement::Left,
				Some(px(280.0)),
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| SourceViewerPanel::new(engine.clone(), window, cx)),
				DockPlacement::Center,
				None,
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| ProgramViewerPanel::new(engine.clone(), window, cx)),
				DockPlacement::Center,
				None,
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| NodeEditorPanel::new(engine.clone(), window, cx)),
				DockPlacement::Center,
				None,
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| InspectorPanel::new(engine.clone(), window, cx)),
				DockPlacement::Right,
				Some(px(320.0)),
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| EffectLibraryPanel::new(engine.clone(), window, cx)),
				DockPlacement::Right,
				None,
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| TimelinePanel::new(engine.clone(), window, cx)),
				DockPlacement::Bottom,
				Some(px(320.0)),
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| HistoryPanel::new(engine.clone(), window, cx)),
				DockPlacement::Bottom,
				None,
				window,
				cx,
			);
			dock.add_panel(
				cx.new(|cx| MulticamPanel::new(engine.clone(), window, cx)),
				DockPlacement::Center,
				None,
				window,
				cx,
			);
		});
		Self { engine, dock, skin }
	}
}

impl<E: AppEngine> Render for AppShell<E> {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		div().size_full().child(self.dock.clone())
	}
}

/// The application entry point: gpui-kit init, theme, i18n, then the main
/// window over the selected engine backend (the real engine by default,
/// the demo with `--mock` or `OAK_ENGINE=mock`).
pub fn run() {
	let mock = std::env::args().any(|arg| arg == "--mock")
		|| std::env::var("OAK_ENGINE").ok().as_deref() == Some("mock");
	gpui_kit::application().run(move |cx| {
		gpui_kit::init(cx);
		Theme::change(ThemeMode::Dark, None, cx);
		i18n::init();
		if mock {
			open_main_window(cx, MockEngine::demo());
		} else {
			open_main_window(cx, SdkEngine::new());
		}
	});
}

/// Opens the main window over a fresh engine entity.
fn open_main_window<E: AppEngine>(cx: &mut App, engine: E) {
	gpui_kit::open_window(
		WindowOptions {
			window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
				None,
				size(px(1440.0), px(900.0)),
				cx,
			))),
			titlebar: Some(TitlebarOptions {
				title: Some("Oak".into()),
				..Default::default()
			}),
			..Default::default()
		},
		cx,
		|window, cx| {
			let engine = cx.new(|_| engine);
			cx.new(|cx| AppShell::new(engine, window, cx))
		},
	)
	.expect("open the main window");
}

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

//! The effect library panel (效果库): every effect type the engine can add
//! to a clip's chain, as a grouped, searchable list. Double-clicking an
//! entry appends the effect to the selected clip's effect chain (the
//! undoable [`AppEngine::add_effect`]; the insertion index is clamped to
//! the chain end by the backend). Every group header collapses and expands
//! its entries; the collapsed set is persisted across sessions.
//!
//! Built entirely from gpui-kit components: the search field is the styled
//! [`Input`] bound to an [`InputState`], the group headers carry
//! [`Icon`] chevrons, and the panel itself docks through the gpui-kit
//! [`Panel`] / [`BasePanel`] pattern.

use std::collections::HashSet;

use gpui_kit::base::input::InputState;
use gpui_kit::colors::DefaultColors as _;
use gpui_kit::component::dock::{BasePanel, Panel, PanelEvent};
use gpui_kit::component::input::{Input, InputEvent};
use gpui_kit::component::{Icon, IconName, Sizable as _, h_flex, v_flex};
use gpui_kit::{
	App, AppContext as _, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable,
	FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
	StatefulInteractiveElement as _, Styled as _, Window, div,
};
use oak_engine_rust_sdk::state::{config_get, config_set};

use crate::engine::{AppEngine, EffectEntry};
use crate::i18n;

/// The config key the collapsed groups are persisted under (a comma
/// separated list of group keys — the keys [`EffectEntry::group`] holds,
/// not the display labels).
const COLLAPSED_CONFIG_KEY: &str = "EffectLibraryCollapsed";

/// The drag payload a library row hands to the effect stack / node editor
/// drop targets: the effect's type id (what `add_effect` takes back) and
/// its display name (shown on the drag ghost).
#[derive(Clone)]
pub struct LibraryEffectDrag {
	pub type_id: SharedString,
	pub name: SharedString,
}

/// The effect library panel.
pub struct EffectLibraryPanel<E: AppEngine> {
	engine: Entity<E>,
	focus_handle: FocusHandle,
	/// The search box state: live-filters the list by name / type id
	/// (case-insensitive substring).
	search: Entity<InputState>,
	/// The group keys whose entries are collapsed away. The headers stay
	/// rendered (and clickable) so a collapsed group can be reopened.
	collapsed: HashSet<String>,
}

impl<E: AppEngine> EffectLibraryPanel<E> {
	/// Builds the panel over `engine`'s addable-effect table.
	pub fn new(engine: Entity<E>, window: &mut Window, cx: &mut Context<Self>) -> Self {
		// Re-read the effect table whenever the engine notifies (the table
		// itself is static, but the selection hint depends on the target).
		cx.observe(&engine, |_this, _engine, cx| cx.notify())
			.detach();
		let search = cx.new(|cx| InputState::new(window, cx));
		// Typing in the search box re-filters the list.
		cx.subscribe_in(
			&search,
			window,
			|_this, _state, event: &InputEvent, _window, cx| {
				if matches!(event, InputEvent::Change) {
					cx.notify();
				}
			},
		)
		.detach();
		Self {
			engine,
			focus_handle: cx.focus_handle(),
			search,
			collapsed: load_collapsed(),
		}
	}
}

/// Reads the persisted collapsed groups (a missing or malformed value
/// simply starts with every group expanded).
fn load_collapsed() -> HashSet<String> {
	parse_collapsed(&config_get(None, COLLAPSED_CONFIG_KEY).unwrap_or_default())
}

/// Parses the persisted comma separated key list; blanks and padding
/// drop, so a hand-edited value cannot produce a phantom group.
fn parse_collapsed(raw: &str) -> HashSet<String> {
	raw.split(',')
		.map(str::trim)
		.filter(|key| !key.is_empty())
		.map(str::to_string)
		.collect()
}

/// Serializes the collapsed set for the config: sorted and comma
/// separated, so an unchanged set persists an unchanged string.
fn format_collapsed(collapsed: &HashSet<String>) -> String {
	let mut keys: Vec<&str> = collapsed.iter().map(String::as_str).collect();
	keys.sort_unstable();
	keys.join(",")
}

/// Writes the collapsed set back to the config (the config store is the
/// source of truth, read once at panel construction).
fn store_collapsed(collapsed: &HashSet<String>) {
	let _ = config_set(None, COLLAPSED_CONFIG_KEY, &format_collapsed(collapsed));
}

/// Flips the collapsed state of `group_key`, returning the new state
/// (`true` = collapsed).
fn toggle_collapsed(collapsed: &mut HashSet<String>, group_key: &str) -> bool {
	if collapsed.remove(group_key) {
		false
	} else {
		collapsed.insert(group_key.to_string());
		true
	}
}

/// Whether `entry` matches the trimmed, lowercased search query.
fn matches_query(entry: &EffectEntry, query: &str) -> bool {
	query.is_empty()
		|| entry.name.to_lowercase().contains(query)
		|| entry.type_id.to_lowercase().contains(query)
}

/// Whether `entry`'s row is collapsed away. The collapse only narrows the
/// unfiltered list: a search looks through every group, collapsed or not.
fn hidden_by_collapse(entry: &EffectEntry, query: &str, collapsed: &HashSet<String>) -> bool {
	query.is_empty()
		&& entry
			.group
			.as_deref()
			.is_some_and(|group| collapsed.contains(group))
}

/// The display label of a group key: the built-in category keys translate
/// through the language packs; anything else (OFX sub-categories) shows
/// as-is.
fn group_label(group: &str) -> String {
	let key = match group {
		"color" => Some("effect_library.group.color"),
		"filter" => Some("effect_library.group.filter"),
		"distort" => Some("effect_library.group.distort"),
		"keying" => Some("effect_library.group.keying"),
		"generator" => Some("effect_library.group.generator"),
		"math" => Some("effect_library.group.math"),
		"time" => Some("effect_library.group.time"),
		"channel" => Some("effect_library.group.channel"),
		"transition" => Some("effect_library.group.transition"),
		"general" => Some("effect_library.group.general"),
		_ => None,
	};
	match key {
		Some(key) => i18n::tr(key).to_string(),
		None => group.to_string(),
	}
}

impl<E: AppEngine> Render for EffectLibraryPanel<E> {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let colors = cx.default_colors().clone();
		let effects = self.engine.read(cx).addable_effects();
		let query = self.search.read(cx).value().trim().to_lowercase();

		let mut list = v_flex()
			.id("effect-library-list")
			.flex_1()
			.min_h_0()
			.gap_1()
			.p_2()
			.overflow_y_scroll();

		// Built-in effects group by their category (color / filter /
		// distort / keying / generator / math / general, see
		// `group_label`); OpenFX plugin entries group by their
		// sub-category. The engine table arrives sorted (built-ins first,
		// then groups and names alphabetically); the search box
		// live-filters by name / type id and always searches collapsed
		// groups too.
		let mut last_group: Option<Option<String>> = None;
		for entry in &effects {
			if !matches_query(entry, &query) {
				continue;
			}
			let group_key = entry.group.clone();
			if last_group.as_ref() != Some(&group_key) {
				last_group = Some(group_key.clone());
				let key = group_key.clone().unwrap_or_default();
				let label = match &entry.group {
					Some(group) => group_label(group),
					None => i18n::tr("effect_library.group.builtin").to_string(),
				};
				let collapsed = self.collapsed.contains(&key);
				list = list.child(group_header(
					&key,
					&label,
					collapsed,
					cx.listener({
						let key = key.clone();
						move |this, _event: &ClickEvent, _window, cx| {
							toggle_collapsed(&mut this.collapsed, &key);
							store_collapsed(&this.collapsed);
							cx.notify();
						}
					}),
				));
			}
			if hidden_by_collapse(entry, &query, &self.collapsed) {
				continue;
			}
			let engine = self.engine.clone();
			let row_id = entry.type_id.clone();
			let name = entry.name.clone();
			let type_id = entry.type_id.clone();
			let drag_payload = LibraryEffectDrag {
				type_id: SharedString::from(entry.type_id.clone()),
				name: SharedString::from(entry.name.clone()),
			};
			list = list.child(
				div()
					.id(SharedString::from(format!("effect-library-{type_id}")))
					.debug_selector(move || format!("effect-library-row-{row_id}"))
					.cursor_pointer()
					.px_2()
					.py_1()
					.rounded_sm()
					.text_sm()
					.text_color(colors.text)
					.hover(|style| style.bg(colors.selected))
					.child(name)
					// Drag the effect onto the inspector's effect stack (or
					// the node editor) to add it there; double-click adds
					// it to the selected clip's chain end.
					.on_drag(drag_payload, |payload, _origin, _window, cx| {
						cx.new(|_cx| EffectDragGhost {
							name: payload.name.clone(),
						})
					})
					.on_click(move |event: &ClickEvent, _window, cx| {
						// Double-click appends the effect to the selected
						// clip's chain; the backend clamps the index to
						// the chain end.
						if event.click_count() == 2 {
							let type_id = type_id.clone();
							engine.update(cx, |engine, cx| {
								if let Err(err) = engine.add_effect(usize::MAX, &type_id, cx) {
									eprintln!("[effect library] add effect failed: {err}");
								}
							});
						}
					}),
			);
		}

		v_flex()
			.size_full()
			.child(
				div()
					.flex_shrink_0()
					.p_2()
					.border_b_1()
					.border_color(colors.border)
					.child(
						Input::new(&self.search)
							.small()
							.cleanable(true)
							.prefix(Icon::new(IconName::Search).small()),
					),
			)
			.child(list)
			.child(
				div()
					.flex_shrink_0()
					.px_2()
					.py_1()
					.border_t_1()
					.border_color(colors.border)
					.text_xs()
					.text_color(colors.disabled)
					.child(i18n::tr("effect_library.hint")),
			)
	}
}

/// The header row of a group: a muted semibold line with a chevron icon
/// that collapses and expands the group's entries. `group_key` is the raw
/// key ([`EffectEntry::group`], also the element id suffix); `label` is
/// its display string.
fn group_header(
	group_key: &str,
	label: &str,
	collapsed: bool,
	on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
	h_flex()
		.id(SharedString::from(format!(
			"effect-library-group-{group_key}"
		)))
		.debug_selector({
			let key = group_key.to_string();
			move || format!("effect-library-group-row-{key}")
		})
		.cursor_pointer()
		.pt_2()
		.pb_1()
		.px_2()
		.items_center()
		.gap_1()
		.text_xs()
		.font_weight(FontWeight(600.0))
		.child(
			Icon::new(if collapsed {
				IconName::ChevronRight
			} else {
				IconName::ChevronDown
			})
			.small(),
		)
		.child(label.to_string())
		.on_click(on_toggle)
}

/// The drag ghost shown under the pointer while an effect is dragged out
/// of the library (a small floating label with the effect name).
struct EffectDragGhost {
	name: SharedString,
}

impl Render for EffectDragGhost {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		let colors = cx.default_colors().clone();
		div()
			.px_2()
			.py_1()
			.rounded_sm()
			.border_1()
			.border_color(colors.border)
			.bg(colors.container)
			.text_sm()
			.text_color(colors.text)
			.child(self.name.clone())
	}
}

impl<E: AppEngine> EventEmitter<PanelEvent> for EffectLibraryPanel<E> {}

impl<E: AppEngine> Focusable for EffectLibraryPanel<E> {
	fn focus_handle(&self, _cx: &App) -> FocusHandle {
		self.focus_handle.clone()
	}
}

impl<E: AppEngine> BasePanel for EffectLibraryPanel<E> {
	fn panel_name(&self) -> &'static str {
		"effect_library"
	}
}

impl<E: AppEngine> Panel for EffectLibraryPanel<E> {
	fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		i18n::tr("panel.effect_library")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::engine::MockEngine;
	use gpui_kit::{Modifiers, Point, TestAppContext, VisualTestContext, px, size};

	/// Serializes the tests that read or write the process-global
	/// collapsed-group key, and starts them from a clean value (a
	/// developer's own persisted choice would otherwise hide rows from
	/// the render tests).
	fn collapsed_config_lock() -> std::sync::MutexGuard<'static, ()> {
		static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
		let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
		store_collapsed(&HashSet::new());
		guard
	}

	/// A one-entry table row for the pure collapse logic.
	fn entry(group: Option<&str>) -> EffectEntry {
		EffectEntry {
			type_id: "oak:testeffect".to_string(),
			name: "Test Effect".to_string(),
			group: group.map(str::to_string),
		}
	}

	/// The row selector `debug_bounds` looks up (leaked: it needs a
	/// `&'static str` and tests are process-lifetime).
	fn row_selector(type_id: &str) -> &'static str {
		Box::leak(format!("effect-library-row-{type_id}").into_boxed_str())
	}

	/// The collapse toggle and the collapsed-row predicate — a collapsed
	/// group hides its rows, but a search still looks inside it.
	#[test]
	fn collapsed_rows_toggle_and_stay_searchable() {
		let color = entry(Some("color"));
		let mut collapsed = HashSet::new();

		assert!(!hidden_by_collapse(&color, "", &collapsed));
		assert!(
			toggle_collapsed(&mut collapsed, "color"),
			"first toggle collapses"
		);
		assert!(hidden_by_collapse(&color, "", &collapsed));
		assert!(
			!hidden_by_collapse(&color, "blur", &collapsed),
			"a search looks through collapsed groups"
		);
		assert!(
			!hidden_by_collapse(&color, "", &HashSet::new()),
			"an unlisted group stays open"
		);
		assert!(
			!toggle_collapsed(&mut collapsed, "color"),
			"second toggle expands"
		);
		assert!(collapsed.is_empty());
	}

	/// The search predicate: name or type id, case-folded substring.
	#[test]
	fn query_matches_name_and_type_id() {
		let color = entry(Some("color"));
		assert!(matches_query(&color, ""));
		assert!(matches_query(&color, "test"));
		assert!(matches_query(&color, "oak:test"));
		assert!(!matches_query(&color, "blur"));
	}

	/// The persisted form round-trips: sorted comma separated keys, with
	/// padding and blank entries dropped.
	#[test]
	fn collapsed_list_round_trips() {
		let collapsed: HashSet<String> =
			["keying", "color"].iter().map(|k| k.to_string()).collect();
		let raw = format_collapsed(&collapsed);
		assert_eq!(raw, "color,keying");
		assert_eq!(parse_collapsed(&raw), collapsed);
		assert_eq!(parse_collapsed(&format!(" {raw} , ")), collapsed);
		assert!(parse_collapsed("").is_empty());
		assert!(parse_collapsed(" , ").is_empty());
	}

	/// The panel's storage path: the collapsed set survives a store/load
	/// cycle through the process config store.
	#[test]
	fn collapsed_state_persists_in_the_config() {
		let _guard = collapsed_config_lock();
		assert!(
			load_collapsed().is_empty(),
			"the lock starts from a clear key"
		);
		let collapsed: HashSet<String> = ["distort", "general"]
			.iter()
			.map(|k| k.to_string())
			.collect();
		store_collapsed(&collapsed);
		assert_eq!(load_collapsed(), collapsed);
		store_collapsed(&HashSet::new());
		assert!(load_collapsed().is_empty());
	}

	/// The panel renders one row per addable effect of the engine.
	#[gpui_kit::test]
	async fn lists_every_addable_effect(cx: &mut TestAppContext) {
		let _guard = collapsed_config_lock();
		cx.update(|cx| {
			gpui_kit::init(cx);
			cx.init_colors();
		});
		let window = cx.open_window(size(px(400.0), px(600.0)), |window, cx| {
			let engine = cx.new(|_| MockEngine::demo());
			EffectLibraryPanel::new(engine, window, cx)
		});
		cx.run_until_parked();
		let cx = VisualTestContext::from_window(window.into(), cx).into_mut();

		let expected = MockEngine::demo_effects();
		assert!(!expected.is_empty());
		for entry in &expected {
			let type_id = &entry.type_id;
			assert!(
				cx.debug_bounds(row_selector(type_id)).is_some(),
				"effect row {type_id} rendered"
			);
		}
	}

	/// A collapsed group keeps its header (so it can be reopened) but
	/// renders none of its rows; the groups below it are unaffected.
	#[gpui_kit::test]
	async fn collapsing_a_group_hides_its_rows(cx: &mut TestAppContext) {
		let _guard = collapsed_config_lock();

		let entries = MockEngine::demo_effects();
		let group = entries
			.first()
			.and_then(|entry| entry.group.clone())
			.expect("the demo table is grouped");
		let other = entries
			.iter()
			.find(|entry| entry.group.as_deref() != Some(group.as_str()))
			.expect("the table spreads over more than one group")
			.clone();

		cx.update(|cx| {
			gpui_kit::init(cx);
			cx.init_colors();
		});
		let window = cx.open_window(size(px(400.0), px(600.0)), |window, cx| {
			let engine = cx.new(|_| MockEngine::demo());
			let mut panel = EffectLibraryPanel::new(engine, window, cx);
			// The state the toggle writes (and the config reloads on the
			// next start), without touching the process-global key.
			panel.collapsed.insert(group.clone());
			panel
		});
		cx.run_until_parked();
		let cx = VisualTestContext::from_window(window.into(), cx).into_mut();

		assert!(
			cx.debug_bounds(Box::leak(
				format!("effect-library-group-row-{group}").into_boxed_str()
			))
			.is_some(),
			"the collapsed group keeps its header"
		);
		for entry in entries
			.iter()
			.filter(|entry| entry.group.as_deref() == Some(group.as_str()))
		{
			assert!(
				cx.debug_bounds(row_selector(&entry.type_id)).is_none(),
				"collapsed row {} is hidden",
				entry.type_id
			);
		}
		assert!(
			cx.debug_bounds(row_selector(&other.type_id)).is_some(),
			"the next group still renders its rows"
		);
	}

	/// Clicking a group header is what writes the state: the group's rows
	/// leave the render and the collapsed key lands in the persisted
	/// config.
	#[gpui_kit::test]
	async fn clicking_a_group_header_collapses_and_persists_it(cx: &mut TestAppContext) {
		let _guard = collapsed_config_lock();

		let entries = MockEngine::demo_effects();
		let group = entries
			.first()
			.and_then(|entry| entry.group.clone())
			.expect("the demo table is grouped");
		let row = entries
			.iter()
			.find(|entry| entry.group.as_deref() == Some(group.as_str()))
			.expect("the group has rows")
			.type_id
			.clone();

		cx.update(|cx| {
			gpui_kit::init(cx);
			cx.init_colors();
		});
		let window = cx.open_window(size(px(400.0), px(600.0)), |window, cx| {
			let engine = cx.new(|_| MockEngine::demo());
			EffectLibraryPanel::new(engine, window, cx)
		});
		cx.run_until_parked();
		let visual = VisualTestContext::from_window(window.into(), cx).into_mut();
		visual.update(|window, cx| {
			window.draw(cx).clear(cx);
		});

		let header = visual
			.debug_bounds(Box::leak(
				format!("effect-library-group-row-{group}").into_boxed_str(),
			))
			.expect("the group header is painted");
		let center = Point::new(
			header.origin.x + header.size.width * 0.5,
			header.origin.y + header.size.height * 0.5,
		);
		assert!(
			visual.debug_bounds(row_selector(&row)).is_some(),
			"the row starts visible"
		);

		visual.simulate_click(center, Modifiers::default());
		visual.update(|window, cx| {
			window.draw(cx).clear(cx);
		});

		assert!(
			visual.debug_bounds(row_selector(&row)).is_none(),
			"the clicked group collapsed"
		);
		assert!(
			load_collapsed().contains(&group),
			"the collapse was persisted under {COLLAPSED_CONFIG_KEY}"
		);
	}
}

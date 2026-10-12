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

//! The demo engine: static tables and in-memory state, so the shell runs
//! (and tests) without the engine library. Selected with `--mock` or
//! `OAK_ENGINE=mock`.

use gpui_kit::Context;

use super::{AppEngine, EffectEntry, EngineError};

/// The demo backend: a static effect table and a record of the effects
/// the user added (so double-clicking the library is observable).
pub struct MockEngine {
	/// The effects added through [`AppEngine::add_effect`], in order
	/// (tests and the demo UI read it back).
	pub added_effects: Vec<String>,
}

impl MockEngine {
	/// The demo backend with the built-in demo effect table.
	pub fn demo() -> Self {
		Self {
			added_effects: Vec::new(),
		}
	}

	/// The demo effect table: a representative slice of the built-in
	/// categories plus an OpenFX-looking group, sorted the way the
	/// library shows them (built-ins first; groups and names
	/// alphabetically).
	pub fn demo_effects() -> Vec<EffectEntry> {
		let mut effects = Vec::new();
		let mut push = |type_id: &str, name: &str, group: Option<&str>| {
			effects.push(EffectEntry {
				type_id: type_id.to_string(),
				name: name.to_string(),
				group: group.map(str::to_string),
			});
		};
		push("oak.color.brightness", "Brightness / Contrast", Some("color"));
		push("oak.color.colorcorrector", "Color Corrector", Some("color"));
		push("oak.color.saturation", "Saturation", Some("color"));
		push("oak.filter.blur", "Gaussian Blur", Some("filter"));
		push("oak.filter.sharpen", "Sharpen", Some("filter"));
		push("oak.distort.lens", "Lens Distortion", Some("distort"));
		push("oak.keying.chroma", "Chroma Key", Some("keying"));
		push("oak.generator.solid", "Solid Color", Some("generator"));
		push("oak.general.transform", "Transform", Some("general"));
		push("com.example.ofx.vignette", "Vignette", Some("OFX"));
		effects
	}
}

impl AppEngine for MockEngine {
	fn addable_effects(&self) -> Vec<EffectEntry> {
		Self::demo_effects()
	}

	fn add_effect(
		&mut self,
		_index: usize,
		type_id: &str,
		cx: &mut Context<Self>,
	) -> Result<(), EngineError> {
		if !self.addable_effects().iter().any(|e| e.type_id == type_id) {
			return Err(EngineError(format!("unknown effect type {type_id}")));
		}
		self.added_effects.push(type_id.to_string());
		cx.notify();
		Ok(())
	}

	fn project_label(&self) -> String {
		"Demo Project".to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// The demo table is non-empty, grouped, and unique by type id — the
	/// effect library's render tests rely on all three.
	#[test]
	fn demo_table_is_grouped_and_unique() {
		let effects = MockEngine::demo_effects();
		assert!(effects.len() >= 8);
		assert!(effects.iter().any(|e| e.group.is_some()));
		let mut ids: Vec<&str> = effects.iter().map(|e| e.type_id.as_str()).collect();
		ids.sort_unstable();
		ids.dedup();
		assert_eq!(ids.len(), effects.len(), "type ids are unique");
	}
}

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

//! The real engine backend: every call goes through the
//! `oak-engine-rust-sdk` C-ABI client into the `oak_engine` dylib. The SDK
//! surface is still growing; operations it does not expose yet fail with a
//! descriptive [`EngineError`] rather than silently doing nothing.

use gpui_kit::Context;

use oak_engine_rust_sdk::node::Project;
use oak_engine_rust_sdk::ofx;

use super::{AppEngine, EffectEntry, EngineError};

/// The SDK-backed engine. Holds the open project (if any); project
/// lifecycle goes through [`SdkEngine::open`] / [`SdkEngine::close`].
pub struct SdkEngine {
	project: Option<Project>,
}

impl SdkEngine {
	/// An engine with no project open.
	pub fn new() -> Self {
		Self { project: None }
	}

	/// Opens the project at `path` (`.ove`), replacing the current one.
	/// Returns the SDK error string on failure.
	pub fn open(&mut self, path: &str, cx: &mut Context<Self>) -> Result<(), EngineError> {
		match Project::load(path) {
			Some(project) => {
				self.project = Some(project);
				cx.notify();
				Ok(())
			}
			None => Err(EngineError(format!("failed to open project {path}"))),
		}
	}

	/// Closes the current project.
	pub fn close(&mut self, cx: &mut Context<Self>) {
		self.project = None;
		cx.notify();
	}

	/// The open project, if any.
	pub fn project(&self) -> Option<&Project> {
		self.project.as_ref()
	}
}

impl Default for SdkEngine {
	fn default() -> Self {
		Self::new()
	}
}

impl AppEngine for SdkEngine {
	fn addable_effects(&self) -> Vec<EffectEntry> {
		// The SDK exposes no built-in effect factory table yet; what it
		// does expose is the scanned OpenFX plugin list (`ofx::scan` runs
		// at startup). Plugins arrive ungrouped — the SDK does not carry
		// the OFX sub-category — so they land in the library's built-in
		// bucket for now.
		let mut effects: Vec<EffectEntry> = ofx::plugins()
			.into_iter()
			.map(|plugin| EffectEntry {
				type_id: plugin.identifier,
				name: plugin.label,
				group: None,
			})
			.collect();
		effects.sort_by(|a, b| a.name.cmp(&b.name));
		effects
	}

	fn add_effect(
		&mut self,
		_index: usize,
		type_id: &str,
		_cx: &mut Context<Self>,
	) -> Result<(), EngineError> {
		// Effect chains are not part of the engine's C ABI yet. Failing
		// loudly keeps the UI honest: the panel logs the error instead of
		// pretending the effect was added.
		Err(EngineError(format!(
			"adding effect {type_id} is not exposed by the engine SDK yet"
		)))
	}

	fn project_label(&self) -> String {
		self.project
			.as_ref()
			.map(|p| {
				let name = p.name();
				if name.is_empty() {
					"Untitled".to_string()
				} else {
					name
				}
			})
			.unwrap_or_else(|| "Untitled".to_string())
	}
}

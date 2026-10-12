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

//! The engine seam: every panel talks to the engine through the
//! [`AppEngine`] trait, never to the SDK directly. Two backends exist:
//!
//! * [`SdkEngine`] — the real engine, driven through the
//!   `oak-engine-rust-sdk` C-ABI client (the engine lives in the
//!   `oak_engine` dylib);
//! * [`MockEngine`] — the demo backend: static tables and in-memory state,
//!   so the UI runs (and tests) without the engine library.
//!
//! The trait is deliberately narrow: it covers only what the SDK can do
//! today. When the engine's C ABI grows (effect chains, selection, viewer
//! frames), the trait grows with it and the panels follow without a
//! rewrite.

mod mock;
mod sdk;

pub use mock::MockEngine;
pub use sdk::SdkEngine;

use gpui_kit::Context;

/// One effect type the engine can add to a clip's chain (the effect
/// library's row data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectEntry {
	/// The stable type identifier (a factory type id or an OFX plugin
	/// identifier) — what `add_effect` takes back.
	pub type_id: String,
	/// The display name.
	pub name: String,
	/// The group the library lists the entry under (a built-in category
	/// key like `color` / `filter`, or an OFX sub-category). `None` lands
	/// in the library's built-in bucket.
	pub group: Option<String>,
}

/// An engine operation failure. The SDK reports errors as plain strings;
/// the seam keeps them that way (the UI shows them verbatim in logs and
/// dialogs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError(pub String);

impl std::fmt::Display for EngineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0)
	}
}

impl std::error::Error for EngineError {}

/// The engine gateway the panels are written against.
///
/// Implemented by entities (`Entity<impl AppEngine>`), so reads happen
/// through `cx` and mutations notify observers — a panel that renders
/// engine data subscribes once with `cx.observe` and re-renders on every
/// mutation.
pub trait AppEngine: 'static + Sized {
	/// Every effect type the engine can add to a clip's chain, sorted the
	/// way the library shows them (built-ins first, then groups and names
	/// alphabetically).
	fn addable_effects(&self) -> Vec<EffectEntry>;

	/// Adds the effect `type_id` to the selected clip's chain at `index`
	/// (clamped to the chain end by the backend; `usize::MAX` appends).
	///
	/// The engine SDK does not expose effect chains yet, so the SDK
	/// backend reports an error here; the mock records the addition so
	/// the interaction is testable.
	fn add_effect(
		&mut self,
		index: usize,
		type_id: &str,
		cx: &mut Context<Self>,
	) -> Result<(), EngineError>;

	/// The window-title label: the project name, or a placeholder while
	/// no project is open.
	fn project_label(&self) -> String;
}

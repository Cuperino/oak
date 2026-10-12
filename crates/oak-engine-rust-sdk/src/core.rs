//! oak-core's SDK surface (the engine's `oak_core_*` ABI), one stop.
//!
//! oak-core is the shared foundation crate: value types (rational time,
//! video params, pixel formats), the process-wide config store, color
//! pipeline state, color math, file utilities, LUT plumbing, and the
//! cooperative cancellation token. In this SDK its functions live in
//! topic modules; this module re-exports them so consumers can use
//! `core::` paths exclusively:
//!
//! | topic | module | contents |
//! |---|---|---|
//! | value types | [`types`] | `Rational`/`TimeRange` arithmetic + timestamp conversion, `VideoParams`, `PixelFormat` |
//! | process state | [`state`] | the config store, color pipeline settings, display ICC probing, `CancelAtom` |
//! | utilities | [`utils`] | color math (transfer functions, primaries, chromatic adaptation), file helpers, LUT display plumbing |
//! | color processing | [`color`] | `ColorProcessor`/`ColorTransform` handles (OCIO-backed conversion) |
//!
//! Platform note: the display-ICC entry points are gated per platform
//! (`windows`/`linux`) on both sides of the ABI.

pub use crate::color;
pub use crate::state;
pub use crate::types;
pub use crate::utils;

#[cfg(test)]
mod tests {
	/// The umbrella paths resolve and reach the engine.
	#[test]
	fn umbrella_paths_work() {
		use super::*;
		let r = types::Rational::new(30000, 1001);
		assert!((r.to_f64() - 29.97).abs() < 1e-3);
		assert!(!state::color_working_space().is_empty());
		assert!(utils::primaries_srgb().red.x > 0.3);
		let atom = state::CancelAtom::new();
		assert!(!atom.is_cancelled());
		// Pass-through carries no OCIO transform (identity conversion).
		let proc = color::ColorProcessor::pass_through();
		assert_eq!(proc.convert_color([0.1, 0.5, 0.9, 1.0]), [0.1, 0.5, 0.9, 1.0]);
	}
}

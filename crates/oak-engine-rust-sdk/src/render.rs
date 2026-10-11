//! Safe wrapper over the engine's render C ABI (`oak_render_*`):
//! render a project's viewer frame (CPU, F32 RGBA) and manage the disk
//! cache.
//!
//! [`render_frame_f32`] is the headless frame-grab a functional plugin
//! builds on. OFX effect nodes on the graph need the plugin executor
//! installed first ([`crate::ofx::scan`] +
//! [`crate::ofx::install_render_executor`]).

use std::ffi::c_void;

use crate::codec::string_out;
use crate::node::{NodeId, Project};
use crate::types::Rational;

#[cfg_attr(
	windows,
	link(name = "oak_engine.dll.lib", kind = "dylib", modifiers = "+verbatim")
)]
#[cfg_attr(not(windows), link(name = "oak_engine", kind = "dylib"))]
unsafe extern "C" {
	fn oak_render_frame_f32(
		project: *const c_void,
		viewer: u64,
		time: Rational,
		width: i32,
		height: i32,
		dst: *mut f32,
		dst_capacity: usize,
	) -> i32;
	fn oak_render_disk_cache_path(buf: *mut u8, buf_len: usize) -> i32;
	fn oak_render_disk_cache_size() -> i64;
	fn oak_render_disk_cache_clear() -> bool;
}

/// Renders the frame of the viewer node `viewer` (any node id from the
/// project traversal, typically a sequence) at `time`, returning tight
/// F32 RGBA pixels (`width*height*4` floats). None on failure (unknown
/// viewer, invalid size, render error).
pub fn render_frame_f32(
	project: &Project,
	viewer: NodeId,
	time: Rational,
	size: (u32, u32),
) -> Option<Vec<f32>> {
	let (w, h) = size;
	if w == 0 || h == 0 {
		return None;
	}
	let mut dst = vec![0.0f32; (w * h * 4) as usize];
	let written = unsafe {
		oak_render_frame_f32(
			project.as_raw(),
			viewer.raw(),
			time,
			w as i32,
			h as i32,
			dst.as_mut_ptr(),
			dst.len(),
		)
	};
	if written < 0 {
		return None;
	}
	dst.truncate(written as usize);
	Some(dst)
}

/// The disk cache directory.
pub fn disk_cache_path() -> String {
	string_out(|buf, len| unsafe { oak_render_disk_cache_path(buf, len) }).unwrap_or_default()
}

/// The disk cache size in bytes.
pub fn disk_cache_size() -> Option<i64> {
	let size = unsafe { oak_render_disk_cache_size() };
	(size >= 0).then_some(size)
}

/// Clears the disk cache.
pub fn disk_cache_clear() -> bool {
	unsafe { oak_render_disk_cache_clear() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn render_empty_sequence_frame() {
		let project = Project::new();
		let sequence = project.create_sequence("Seq").expect("sequence");
		let frame = render_frame_f32(&project, sequence, Rational::new(0, 1), (16, 8))
			.expect("an empty sequence renders");
		assert_eq!(frame.len(), 16 * 8 * 4);
		assert!(frame.iter().all(|&v| v == 0.0), "transparent black");
		// An unknown viewer fails closed.
		assert!(
			render_frame_f32(&project, NodeId::INVALID, Rational::new(0, 1), (16, 8)).is_none()
		);
	}

	#[test]
	fn disk_cache_entry_points_are_safe() {
		assert!(!disk_cache_path().is_empty());
		let _ = disk_cache_size();
	}
}

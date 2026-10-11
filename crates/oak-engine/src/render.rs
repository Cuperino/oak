//! C ABI for oak-render (prefix `oak_render_`): render a project's
//! viewer frame (CPU, F32 RGBA) and manage the disk cache.
//!
//! [`oak_render_frame_f32`] is the headless frame-grab a functional
//! plugin builds on ("render.get_frame"): render the graph of any
//! sequence/viewer node at a given time into a caller buffer. OFX
//! effect nodes on the graph need the plugin executor installed first
//! (`oak_ofx_scan` + `oak_ofx_install_render_executor`).
//!
//! The process pool / scheduler / pipeline are host-side
//! infrastructure and intentionally not exported; cancellation uses
//! the existing `oak_core_cancelatom_*` surface.

use oak_core::PixelFormat;
use oak_node::id::NodeId;

use crate::codec::copy_str_out;
use crate::coretypes::OakRational;
use crate::handle::NativeMirror;
use crate::node::ProjectHandle;

/// Renders the frame of the viewer node `viewer` (a node id token, see
/// `oak_project_root`/`oak_node_*` traversal) at `time` into `dst` as
/// tight F32 RGBA (`width*height*4` floats). Returns the float count
/// written, -1 on a null handle, an unknown viewer, invalid sizes, an
/// undersized buffer, or a render failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oak_render_frame_f32(
	project: *const ProjectHandle,
	viewer: u64,
	time: OakRational,
	width: i32,
	height: i32,
	dst: *mut f32,
	dst_capacity: usize,
) -> i32 {
	if project.is_null() || dst.is_null() || width <= 0 || height <= 0 {
		return -1;
	}
	let needed = width as usize * height as usize * 4;
	if dst_capacity < needed {
		return -1;
	}
	let Some(viewer) = NodeId::from_identity(viewer) else {
		return -1;
	};
	let project = unsafe { &*project }.project();
	let texture = match oak_render::eval::render_graph_frame(
		project,
		viewer,
		time.to_native(),
		(width, height),
		PixelFormat::F32,
	) {
		Ok(texture) => texture,
		Err(_) => return -1,
	};
	let Ok(frame) = texture.to_frame() else {
		return -1;
	};
	let params = frame.video_params();
	if params.width != width || params.height != height || params.format != PixelFormat::F32 as i32
	{
		return -1;
	}
	// Copy row by row (the frame's stride may be padded; the output is
	// tight RGBA F32).
	let tight = width as usize * 4 * 4;
	let row = frame.linesize_bytes().max(tight);
	if frame.data.len() < row * (height as usize - 1) + tight {
		return -1;
	}
	let out = unsafe { std::slice::from_raw_parts_mut(dst as *mut u8, tight * height as usize) };
	for y in 0..height as usize {
		out[y * tight..(y + 1) * tight].copy_from_slice(&frame.data[y * row..y * row + tight]);
	}
	needed as i32
}

/// The disk cache directory, snprintf-style.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oak_render_disk_cache_path(buf: *mut u8, buf_len: usize) -> i32 {
	copy_str_out(&oak_render::manager::disk_cache_path(), buf, buf_len)
}

/// The disk cache size in bytes; -1 on failure.
#[unsafe(no_mangle)]
pub extern "C" fn oak_render_disk_cache_size() -> i64 {
	oak_render::manager::disk_cache_size().unwrap_or(-1)
}

/// Clears the disk cache. False on failure.
#[unsafe(no_mangle)]
pub extern "C" fn oak_render_disk_cache_clear() -> bool {
	oak_render::manager::disk_cache_clear().is_ok()
}

#[cfg(test)]
mod tests {
	use super::*;

	/// A fresh sequence renders to transparent black through the C ABI.
	#[test]
	fn render_empty_sequence_frame() {
		unsafe {
			let project = crate::node::oak_project_new();
			assert!(!project.is_null());
			let (n, nl) = ("Seq".as_ptr(), 3);
			let seq = crate::node::oak_sequence_create(project, n, nl);
			assert!(crate::node::oak_node_id_is_valid(seq));

			let (w, h) = (16, 8);
			let mut dst = vec![1.0f32; w * h * 4];
			let written = oak_render_frame_f32(
				project,
				seq,
				OakRational { num: 0, den: 1 },
				w as i32,
				h as i32,
				dst.as_mut_ptr(),
				dst.len(),
			);
			assert_eq!(written, (w * h * 4) as i32);
			assert!(
				dst.iter().all(|&v| v == 0.0),
				"an empty sequence is transparent black"
			);

			// Fail closed: unknown viewer, bad sizes, undersized buffer.
			assert_eq!(
				oak_render_frame_f32(
					project,
					u64::MAX,
					OakRational { num: 0, den: 1 },
					w as i32,
					h as i32,
					dst.as_mut_ptr(),
					dst.len()
				),
				-1
			);
			assert_eq!(
				oak_render_frame_f32(
					project,
					seq,
					OakRational { num: 0, den: 1 },
					0,
					h as i32,
					dst.as_mut_ptr(),
					dst.len()
				),
				-1
			);
			assert_eq!(
				oak_render_frame_f32(
					project,
					seq,
					OakRational { num: 0, den: 1 },
					w as i32,
					h as i32,
					dst.as_mut_ptr(),
					4
				),
				-1
			);
			crate::node::oak_project_release(project);
		}
	}

	#[test]
	fn disk_cache_entry_points_are_safe() {
		let mut buf = [0u8; 1024];
		let n = unsafe { oak_render_disk_cache_path(buf.as_mut_ptr(), buf.len()) };
		assert!(n > 0);
		// Size/clear are environment-dependent; they must not panic and
		// report a coherent shape.
		let size = oak_render_disk_cache_size();
		assert!(size >= -1);
	}
}

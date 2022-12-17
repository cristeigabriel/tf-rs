//! Hooks module

#![allow(non_camel_case_types)]

use crate::{ctx::Context, try_acquire_ctx_locked};
use detour::static_detour;
use game::types::{
    chl_client::CHLClientIndices, mat_system_surface::MatSystemSurfaceIndices, vftable::VfTable,
};
use shared::GenericErrOr;
use std::ffi::c_int;

/// Type for hooks errors
pub(super) type HooksErrOr<T> = GenericErrOr<T>;

static_detour! {
    // `static_detour` doesn't support type aliases
    // so we have to redefine the types for transmutations
    static FrameStageNotifyHook: unsafe extern "stdcall" fn(c_int) -> c_int;
    static PaintTraverseHook: unsafe extern "thiscall" fn(*const usize, c_int) -> c_int;
}

// Above
type FrameStageNotify_t = unsafe extern "stdcall" fn(c_int) -> c_int;
type PaintTraverse_t = unsafe extern "thiscall" fn(*const usize, c_int) -> c_int;

fn frame_stage_notify(stage: c_int) -> c_int {
    unsafe { FrameStageNotifyHook.call(stage) }
}

fn paint_traverse(this: *const usize, panel: c_int) -> c_int {
    unsafe { PaintTraverseHook.call(this, panel) }
}

pub(super) fn install(ctx: &Context) -> HooksErrOr<()> {
    // Apply hooks
    unsafe {
        FrameStageNotifyHook
            .initialize(
                ctx.get_chl_client()
                    .get_virtual_function(CHLClientIndices::FrameStageNotify as isize)
                    .transmute::<FrameStageNotify_t>()?,
                frame_stage_notify,
            )?
            .enable()?;

        // You should be able to get to SolvePanels by looking for words such as 'Panel' or
        // 'Paint' in vguimatsystemsurface, find said function, breakpoint it's client.dll
        // version, breakpoint it, go to first entry in vguimatsurface in stackframe,
        // then PaintTraverse is one function after.
        PaintTraverseHook
            .initialize(
                ctx.get_mat_system_surface()
                    .get_virtual_function(MatSystemSurfaceIndices::PaintTraverse as isize)
                    .transmute::<PaintTraverse_t>()?,
                paint_traverse,
            )?
            .enable()?;
    }

    Ok(())
}

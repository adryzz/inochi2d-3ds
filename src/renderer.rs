use citro3d_sys::{
    C3D_Fini, C3D_FrameBegin, C3D_FrameDrawOn, C3D_FrameEnd, C3D_Init, C3D_RenderTarget,
    C3D_RenderTargetClear, C3D_RenderTargetCreate, C3D_RenderTargetSetOutput, C3D_CLEAR_ALL,
    C3D_DEFAULT_CMDBUF_SIZE, C3D_DEPTHTYPE, C3D_FRAME_SYNCDRAW, GX_TRANSFER_IN_FORMAT,
    GX_TRANSFER_OUT_FORMAT, GX_TRANSFER_SCALING,
};
use ctru::services::{gfx::TopScreen3D, gspgpu::FramebufferFormat};
use ctru_sys::{
    gfx3dSide_t, gfxScreen_t, GFX_LEFT, GFX_RIGHT, GFX_TOP, GPU_DEPTHBUF, GPU_RB_DEPTH24_STENCIL8,
    GPU_RB_RGBA8, GX_TRANSFER_FMT_RGBA8, GX_TRANSFER_SCALE_XY,
};

use std::{
    cell::{BorrowError, BorrowMutError},
    num::TryFromIntError,
};
use thiserror::Error;

const BUF_WIDTH: i32 = 240 * 2;
const BUF_HEIGHT: i32 = 400 * 2;

pub struct GPURenderer3D<'a> {
    top_screen: TopScreen3D<'a>,
    pub left_target: RenderTarget,
    pub right_target: RenderTarget,
}

impl<'a> GPURenderer3D<'a> {
    pub fn new(top_screen: TopScreen3D<'a>) -> Result<Self, RendererError> {
        let left_format: FramebufferFormat;
        let right_format: FramebufferFormat;
        {
            let (left_screen, right_screen) = top_screen.split();
            left_format = left_screen.framebuffer_format();
            right_format = right_screen.framebuffer_format();
        }

        if !unsafe { C3D_Init(C3D_DEFAULT_CMDBUF_SIZE.try_into()?) } {
            return Err(RendererError::Citro3DInit);
        }

        // creating targets

        let mut flags: u32 = 0;
        // set in format to RGBA
        flags += GX_TRANSFER_IN_FORMAT(GX_TRANSFER_FMT_RGBA8);
        // set out format to RGBA
        flags += GX_TRANSFER_OUT_FORMAT(GX_TRANSFER_FMT_RGBA8);
        // set anti-aliasing to 2X
        flags += GX_TRANSFER_SCALING(GX_TRANSFER_SCALE_XY);

        let left_target = RenderTarget::new(
            BUF_WIDTH,
            BUF_HEIGHT,
            left_format,
            GPU_RB_DEPTH24_STENCIL8,
            flags,
            GFX_TOP,
            GFX_LEFT,
        )?;
        let right_target = RenderTarget::new(
            BUF_WIDTH,
            BUF_HEIGHT,
            right_format,
            GPU_RB_DEPTH24_STENCIL8,
            flags,
            GFX_TOP,
            GFX_RIGHT,
        )?;

        return Ok(Self {
            top_screen,
            left_target,
            right_target,
        });
    }

    pub fn clear_all_targets(&mut self) {
        self.left_target.clear();
        self.right_target.clear();
    }

    pub fn clear_all_targets_with_color(&mut self, color: u32) {
        self.left_target.clear_with_color(color);
        self.right_target.clear_with_color(color);
    }

    pub fn begin_frame(&self) {
        unsafe {
            C3D_FrameBegin(/*C3D_FRAME_SYNCDRAW*/ 1)
        };
    }

    pub fn end_frame(&self) {
        unsafe {
            C3D_FrameEnd(/*C3D_FRAME_SYNCDRAW*/ 0)
        };
    }
}

impl<'a> Drop for GPURenderer3D<'a> {
    fn drop(&mut self) {
        unsafe { C3D_Fini() };
    }
}

pub struct RenderTarget {
    target: *mut C3D_RenderTarget,
}

impl RenderTarget {
    pub fn new(
        width: i32,
        height: i32,
        format: FramebufferFormat,
        depth: GPU_DEPTHBUF,
        flags: u32,
        screen: gfxScreen_t,
        side: gfx3dSide_t,
    ) -> Result<Self, RendererError> {
        // create target

        let target = unsafe {
            C3D_RenderTargetCreate(
                width,
                height,
                format.into(),
                C3D_DEPTHTYPE {
                    __i: depth.try_into()?,
                },
            )
        };

        // clear target
        unsafe { C3D_RenderTargetClear(target, C3D_CLEAR_ALL, 0, 0) };

        // bind target to display
        unsafe { C3D_RenderTargetSetOutput(target, screen, side, flags) };

        Ok(Self { target: target })
    }

    pub fn clear(&mut self) {
        unsafe { C3D_RenderTargetClear(self.target, C3D_CLEAR_ALL, 0, 0) };
    }

    pub fn clear_with_color(&mut self, color: u32) {
        unsafe { C3D_RenderTargetClear(self.target, C3D_CLEAR_ALL, color, 0) };
    }

    pub fn select(&mut self) {
        unsafe { C3D_FrameDrawOn(self.target) };
    }
}

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Unknown error.")]
    Unknown,

    #[error("Couldn't initialize Citro3D.")]
    Citro3DInit,

    #[error("Couldn't get a handle to the screen.")]
    ScreenError(#[from] BorrowError),

    #[error("Couldn't get a mut handle to the screen.")]
    ScreenMutError(#[from] BorrowMutError),

    #[error("Couldn't convert int.")]
    IntConversionError(#[from] TryFromIntError),
}

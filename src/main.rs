use std::{
    cell::{BorrowError, RefCell, RefMut},
    fs::read,
    num::TryFromIntError,
};

use citro3d_sys::{
    C3D_Init, C3D_RenderTarget, C3D_RenderTargetCreate, C3D_DEFAULT_CMDBUF_SIZE, C3D_DEPTHTYPE, C3D_RenderTargetClear, C3D_CLEAR_ALL, C3D_RenderTargetSetOutput, GX_TRANSFER_IN_FORMAT, GX_TRANSFER_OUT_FORMAT, GX_TRANSFER_SCALING, C3D_Fini,
};
use ctru::{
    prelude::*,
    services::{
        gfx::{Screen, TopScreen, TopScreen3D},
        gspgpu::FramebufferFormat,
        romfs::RomFS,
    },
};
use ctru_sys::{GPU_RB_DEPTH24_STENCIL8, GPU_RB_RGBA8, GX_TRANSFER_FMT_RGBA8, GX_TRANSFER_SCALE_XY, GFX_TOP, GFX_LEFT, GFX_RIGHT};
use inox2d::formats::inp::parse_inp;
use std::cell::BorrowMutError;
use thiserror::Error;

const BUF_WIDTH: i32 = 240 * 2;
const BUF_HEIGHT: i32 = 400 * 2;

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::with_formats(FramebufferFormat::Rgba8, FramebufferFormat::Bgr8, false).expect("Couldn't obtain GFX controller.");
    let mut hid = Hid::init().expect("Couldn't obtain HID controller.");
    let apt = Apt::init().expect("Couldn't obtain APT controller.");
    let mut soc = Soc::init().expect("Couldn't get SOC controller.");
    let _console = Console::init(gfx.bottom_screen.borrow_mut());

    // init RomFS
    let _romfs = RomFS::init().unwrap();

    // open the .inp file from RomFS
    let data = read("romfs:/model.inp").expect("Couldn't read model from RomFS.");

    let model = parse_inp(data.as_slice()).unwrap();

    println!("== Puppet Meta ==\n{}", &model.puppet.meta);

    gfx.top_screen.borrow_mut().set_double_buffering(true);

    let top_screen = TopScreen3D::from(&gfx.top_screen);

    let mut _renderer = GPURenderer3D::new(top_screen);

    while apt.main_loop() {
        //Scan all the inputs. This should be done once for each frame
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }


        // Flush and swap framebuffers
        gfx.flush_buffers();
        gfx.swap_buffers();

        //Wait for VBlank
        gfx.wait_for_vblank();
    }
}

struct GPURenderer3D<'a> {
    top_screen: TopScreen3D<'a>,
    left_target: *mut C3D_RenderTarget,
    right_target: *mut C3D_RenderTarget,
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

        let left_target = unsafe {
            C3D_RenderTargetCreate(
                BUF_WIDTH,
                BUF_HEIGHT,
                left_format.into(),
                C3D_DEPTHTYPE {
                    __i: GPU_RB_DEPTH24_STENCIL8.try_into()?,
                },
            )
        };
        let right_target = unsafe {
            C3D_RenderTargetCreate(
                BUF_WIDTH,
                BUF_HEIGHT,
                right_format.into(),
                C3D_DEPTHTYPE {
                    __i: GPU_RB_DEPTH24_STENCIL8.try_into()?,
                },
            )
        };

        // clearing targets
        unsafe { C3D_RenderTargetClear(left_target, C3D_CLEAR_ALL, 0, 0) };

        unsafe { C3D_RenderTargetClear(right_target, C3D_CLEAR_ALL, 0, 0) };

        let mut flags: u32 = 0;
        // set in format to RGBA
        flags += GX_TRANSFER_IN_FORMAT(GX_TRANSFER_FMT_RGBA8);
        // set out format to RGBA
        flags += GX_TRANSFER_OUT_FORMAT(GX_TRANSFER_FMT_RGBA8);
        // set anti-aliasing to 2X
        flags += GX_TRANSFER_SCALING(GX_TRANSFER_SCALE_XY);

        // bind targets to display
        unsafe { C3D_RenderTargetSetOutput(left_target, GFX_TOP, GFX_LEFT, flags) };

        unsafe { C3D_RenderTargetSetOutput(right_target, GFX_TOP, GFX_RIGHT, flags) };
        
        return Ok(Self {
            top_screen,
            left_target,
            right_target,
        });
    }
}

impl<'a> Drop for GPURenderer3D<'a> {
    fn drop(&mut self) {
        unsafe { C3D_Fini()};
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

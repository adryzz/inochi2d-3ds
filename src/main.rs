use std::{
    cell::{BorrowError},
    fs::read,
    num::TryFromIntError,
};
use ctru::{
    prelude::*,
    services::{
        gfx::{TopScreen3D},
        gspgpu::FramebufferFormat,
        romfs::RomFS,
    },
};
use inox2d::formats::inp::parse_inp;
use std::cell::BorrowMutError;
use thiserror::Error;

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::with_formats(FramebufferFormat::Rgba8, FramebufferFormat::Bgr8, false).expect("Couldn't obtain GFX controller.");
    // RGBA8 is actually ABGR.
    
    let mut hid = Hid::init().expect("Couldn't obtain HID controller.");
    let apt = Apt::init().expect("Couldn't obtain APT controller.");
    let _console = Console::init(gfx.bottom_screen.borrow_mut());

    // init RomFS
    let _romfs = RomFS::init().unwrap();

    // open the .inp file from RomFS
    let data = read("romfs:/model.inp").expect("Couldn't read model from RomFS.");

    let model = parse_inp(data.as_slice()).unwrap();

    println!("== Puppet Meta ==\n{}", &model.puppet.meta);

    //gfx.top_screen.borrow_mut().set_double_buffering(true);

    let top_screen = TopScreen3D::from(&gfx.top_screen);

    let (mut left, mut right) = top_screen.split_mut();


    let left_data = read("romfs:/left").unwrap();
    let right_data = read("romfs:/right").unwrap();

    while apt.main_loop() {
        //Scan all the inputs. This should be done once for each frame
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        let left_buf = left.raw_framebuffer();
        let right_buf = right.raw_framebuffer();

        unsafe {
            left_buf.ptr.copy_from(left_data.as_ptr(), left_data.len());
            right_buf.ptr.copy_from(right_data.as_ptr(), right_data.len());
        }

        // Flush and swap framebuffers
        gfx.flush_buffers();
        gfx.swap_buffers();

        //Wait for VBlank
        gfx.wait_for_vblank();
    }
}

/*struct GPURenderer3D<'a> {
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
                240,
                400,
                left_format.into(),
                C3D_DEPTHTYPE {
                    __i: GPU_RB_DEPTH24_STENCIL8.try_into()?,
                },
            )
        };
        let right_target = unsafe {
            C3D_RenderTargetCreate(
                240,
                400,
                right_format.into(),
                C3D_DEPTHTYPE {
                    __i: GPU_RB_DEPTH24_STENCIL8.try_into()?,
                },
            )
        };

        // clearing targets
        unsafe { C3D_RenderTargetClear(left_target, C3D_CLEAR_ALL, 0, 0) };

        unsafe { C3D_RenderTargetClear(right_target, C3D_CLEAR_ALL, 0, 0) };

        let flags = Flags::default()
        .in_format(color_format.into())
        .out_format(color_format.into());

        // bind targets to display
        unsafe { C3D_RenderTargetSetOutput(left_target, , , ) };
        return Ok(Self {
            top_screen,
            left_target,
            right_target,
        });
    }
}*/

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

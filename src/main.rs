mod renderer;

use ctru::{
    prelude::*,
    services::{
        gfx::{Screen, TopScreen3D},
        gspgpu::FramebufferFormat,
        romfs::RomFS,
    },
};
use inox2d::formats::inp::parse_inp;
use std::{
    fs::read,
};

use crate::renderer::GPURenderer3D;

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::with_formats(FramebufferFormat::Rgba8, FramebufferFormat::Bgr8, false)
        .expect("Couldn't obtain GFX controller.");
    let mut hid = Hid::init().expect("Couldn't obtain HID controller.");
    let apt = Apt::init().expect("Couldn't obtain APT controller.");
    let _soc = Soc::init().expect("Couldn't get SOC controller.");
    let _console = Console::init(gfx.bottom_screen.borrow_mut());

    // init RomFS
    let _romfs = RomFS::init().unwrap();

    // open the .inp file from RomFS
    let data = read("romfs:/model.inp").expect("Couldn't read model from RomFS.");

    let model = parse_inp(data.as_slice()).unwrap();

    println!("== Puppet Meta ==\n{}", &model.puppet.meta);

    gfx.top_screen.borrow_mut().set_double_buffering(true);

    let top_screen = TopScreen3D::from(&gfx.top_screen);

    let mut renderer = GPURenderer3D::new(top_screen).unwrap();

    while apt.main_loop() {
        //Scan all the inputs. This should be done once for each frame
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        renderer.begin_frame();
        renderer.left_target.clear_with_color(0xFF0000FF);
        renderer.left_target.select();

        renderer.right_target.clear_with_color(0x00FF00FF);
        renderer.right_target.select();
        renderer.end_frame();
        // Flush and swap framebuffers
        gfx.flush_buffers();
        gfx.swap_buffers();

        //Wait for VBlank
        gfx.wait_for_vblank();
    }
}

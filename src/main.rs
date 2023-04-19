use std::fs::read;

use ctru::{prelude::*, services::romfs::RomFS};
use inox2d::formats::inp::parse_inp;

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::init().expect("Couldn't obtain GFX controller.");
    let mut hid = Hid::init().expect("Couldn't obtain HID controller.");
    let apt = Apt::init().expect("Couldn't obtain APT controller.");
    let _console = Console::init(gfx.bottom_screen.borrow_mut());

    // init RomFS
    let _romfs = RomFS::init().unwrap();

    // open the .inp file from RomFS
    let data = read("romfs:/model.inp").expect("Couldn't read model from RomFS.");


    let model = parse_inp(data.as_slice()).unwrap();

    println!("== Puppet Meta ==\n{}", &model.puppet.meta);
    println!("== Nodes ==\n{}", &model.puppet.nodes);
    if model.vendors.is_empty() {
        println!("(No Vendor Data)\n");
    } else {
        println!("== Vendor Data ==");
        for vendor in &model.vendors {
            println!("{vendor}");
        }
    }

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

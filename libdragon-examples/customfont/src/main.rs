#![no_std]
#![no_main]

#[allow(unused_imports)]
use libdragon::*;

use display::{BitDepth, FilterOptions, Gamma, Resolution};
use graphics::Graphics;
use sprite::Sprite;

#[no_mangle]
extern "C" fn main() -> ! {
    // enable ISViewer, so eprintln calls are displayed there
    debug::init(debug::FEATURE_LOG_ISVIEWER | debug::FEATURE_LOG_USB);

    // Initialize peripherals
    display::init(
        Resolution::_320x240,
        BitDepth::Bpp16,
        2,
        Gamma::None,
        FilterOptions::Resample,
    );
    dfs::init(None).unwrap_or_else(|e| panic!("Could not initialize filesystem: {:?}", e));
    rdpq::init();
    joypad::init();
    timer::init();

    // Read in the custom font
    let custom_font = Sprite::load(dfs::PathBuf::from("rom:/libdragon-font.sprite")).unwrap();

    // Grab a render buffer and create a Graphics context
    let mut g = Graphics::new(display::get());

    // Fill the screen
    g.fill_screen(0);

    // Set the text output color
    graphics::set_color(0xFFFF_FFFF, 0);

    g.draw_text(20, 20, "Using default font!");

    graphics::set_font_sprite(custom_font);

    g.draw_text(20, 40, "Using custom font!");

    // Force backbuffer flip
    g.finish().show();

    // Wait forever
    loop {}
}

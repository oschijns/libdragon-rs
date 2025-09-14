#![no_std]
#![no_main]
#![feature(offset_of)]

use libdragon::*;

use libdragon::{
    display::{BitDepth, FilterOptions, Gamma, Resolution},
    model64::*,
};

use core_maths::*;

struct App<'a> {
    camera:    Camera,
    frames:    u64,
    animation: u32,
    model:     Model64<'a>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let model = Model64::load("rom:/rust.model64").unwrap();

        let aspect_ratio = (display::get_width() as f64) / (display::get_height() as f64);
        let near_plane = 1.0;
        let far_plane = 50.0;

        gl::MatrixMode(gl::PROJECTION);
        gl::LoadIdentity();
        gl::Frustum(
            -near_plane * aspect_ratio,
            near_plane * aspect_ratio,
            -near_plane,
            near_plane,
            near_plane,
            far_plane,
        );

        gl::MatrixMode(gl::MODELVIEW);
        gl::LoadIdentity();

        let mat_diffuse = [1.0, 1.0, 1.0, 1.0];
        gl::Materialfv(gl::FRONT_AND_BACK, gl::AMBIENT_AND_DIFFUSE, &mat_diffuse);

        gl::Fogf(gl::FOG_START, 5.0);
        gl::Fogf(gl::FOG_END, 20.0);
        gl::Fogfv(gl::FOG_COLOR, &[1.0, 1.0, 1.0, 1.0]);

        gl::Enable(gl::MULTISAMPLE_ARB);

        rspq::profile_start();

        Self {
            camera: Camera {
                distance: -10.0,
                rotation: 0.0,
            },
            frames: 0,
            animation: 3283,
            model,
        }
    }

    fn render(&mut self) {
        let disp = display::get();
        let zbuf = display::get_zbuf();
        rdpq::attach(&disp, Some(&zbuf));

        gl::context_begin();

        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::MatrixMode(gl::MODELVIEW);
        self.camera.transform();

        //gl::Enable(gl::LIGHTING);
        gl::Enable(gl::NORMALIZE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::PushMatrix();
        gl::Rotatef(90.0, 1.0, 0.0, 0.0);
        gl::Enable(gl::COLOR_MATERIAL);
        gl::Color4f(0.0, 0.0, 0.0, 0.5);
        for i in 0..self.model.get_mesh_count() {
            self.model.get_mesh(i).draw();
        }
        gl::Disable(gl::COLOR_MATERIAL);
        gl::PopMatrix();

        //gl::Disable(gl::TEXTURE_2D);
        //gl::Disable(gl::LIGHTING);

        gl::context_end();

        rdpq::detach_show();

        rspq::profile_next_frame();

        self.frames += 1;
        if (self.frames % 60) == 0 {
            rspq::profile_dump();
            rspq::profile_reset();
            eprintln!("frame {}", self.frames);
        }
    }
}

#[no_mangle]
extern "C" fn main() -> ! {
    debug::init(debug::FEATURE_LOG_ISVIEWER | debug::FEATURE_LOG_USB);

    dfs::init(None).unwrap_or_else(|e| panic!("Could not initialize filesystem: {:?}", e));

    display::init(
        Resolution::_320x240,
        BitDepth::Bpp16,
        3,
        Gamma::None,
        FilterOptions::ResampleAntialiasDedither,
    );

    rdpq::init();
    gl::init();

    rdpq::debug_start();

    let mut app = App::new();

    joypad::init();

    let mut shade_model = gl::SMOOTH;
    let mut fog_enabled = false;

    loop {
        joypad::poll();
        let port = joypad::Port::get_port_1();
        let pressed = port.get_buttons_pressed();
        let held = port.get_buttons_held();
        let inputs = port.get_inputs();

        if held.a {
            app.animation += 1;
        }

        if held.b {
            app.animation -= 1;
        }

        if pressed.start {
            eprintln!("{}", app.animation);
        }

        if pressed.r {
            shade_model = if shade_model == gl::SMOOTH {
                gl::FLAT
            } else {
                gl::SMOOTH
            };
            gl::ShadeModel(shade_model);
        }

        if pressed.l {
            fog_enabled = !fog_enabled;
            if fog_enabled {
                gl::Enable(gl::FOG)
            } else {
                gl::Disable(gl::FOG)
            }
        }

        let y = (inputs.stick_y as f32) / 128.0;
        let x = (inputs.stick_x as f32) / 128.0;
        let mag = x * x + y * y;

        if mag.abs() > 0.01 {
            app.camera.distance += y * 0.2;
            app.camera.rotation = app.camera.rotation - x * 1.2;
        }

        app.render();
    }
}

struct Camera {
    distance: f32,
    rotation: f32,
}

impl Camera {
    pub fn transform(&self) {
        gl::LoadIdentity();
        glu::LookAt(0.0, 0.0, -self.distance, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        gl::Rotatef(self.rotation, 0.0, 1.0, 0.0);
    }
}

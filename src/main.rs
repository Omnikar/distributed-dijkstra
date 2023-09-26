mod agent;
mod sim;

use pixels::{PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const SCREEN_DIMS: (u32, u32) = (1440, 900);

fn main() {
    let mut world = sim::World::new();

    world.sites.push(sim::Site {
        pos: [3.0, 3.0],
        kind: 0,
        size: 0.4,
    });
    world.sites.push(sim::Site {
        pos: [7.0, 2.0],
        kind: 1,
        size: 0.4,
    });
    world.sites.push(sim::Site {
        pos: [5.0, 5.0],
        kind: 2,
        size: 0.4,
    });
    world.sites.push(sim::Site {
        pos: [5.0, 8.0],
        kind: 3,
        size: 0.4,
    });
    world.sites.push(sim::Site {
        pos: [8.0, 7.0],
        kind: 4,
        size: 0.4,
    });
    world.sites.push(sim::Site {
        pos: [8.5, 4.5],
        kind: 5,
        size: 0.4,
    });

    world.site_kinds = vec![
        [0xff, 0x00, 0x00],
        [0x00, 0xff, 0x00],
        [0x00, 0x00, 0xff],
        [0xff, 0xff, 0x00],
        [0x00, 0xff, 0xff],
        [0xff, 0x00, 0xff],
    ];

    let n_sites = world.sites.iter().map(|site| site.kind + 1).max().unwrap();
    world.agents.iter_mut().for_each(|agent| {
        agent.state.sites = vec![(f32::INFINITY, false); n_sites];
        agent.state.sites[0].1 = true;
    });

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(SCREEN_DIMS.0, SCREEN_DIMS.1);
        WindowBuilder::new()
            .with_inner_size(size)
            .with_decorations(false)
            .build(&event_loop)
            .expect("WindowBuilder failed")
    };
    if let Some("--hide-cursor") = std::env::args().nth(1).as_deref() {
        window.set_cursor_visible(false);
    }

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(SCREEN_DIMS.0, SCREEN_DIMS.1, surface_texture)
            .build()
            .expect("PixelsBuilder failed")
    };
    pixels.frame_mut().fill(0xff);

    let mut last_loop = std::time::Instant::now();
    event_loop.run(move |_event, _, control_flow| {
        let now = std::time::Instant::now();
        let delta = (now - last_loop).as_secs_f32();
        last_loop = now;

        let frame = pixels.frame_mut();

        frame
            .chunks_mut(4)
            .for_each(|px| px[0..3].copy_from_slice(&[0x1e, 0x1f, 0x2e]));

        world.update(delta);
        world.render(frame, 90.0, 1440);
        pixels.render().unwrap();

        if let ControlFlow::ExitWithCode(code) = control_flow {
            std::process::exit(*code);
        }
    });
}

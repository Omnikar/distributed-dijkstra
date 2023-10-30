mod agent;
mod math;
mod sim;

use pixels::{PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const SCREEN_DIMS: (u32, u32) = (1440, 900);

fn main() {
    let world_f = std::env::args().nth(1);
    let world_f = world_f.as_deref().unwrap_or("scenes/default.ron");
    let world_s = std::fs::read_to_string(world_f).unwrap();
    let mut world: sim::World = ron::from_str(&world_s).unwrap();
    world.agents = sim::World::new().agents;

    let n_sites = world
        .sites
        .iter()
        .map(|site| site.kind + 1)
        .max()
        .unwrap_or(0);
    world.agents.iter_mut().for_each(|agent| {
        agent.state.sites = vec![(f32::INFINITY, true); n_sites];
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
    const FRAME_TIME_MIN: std::time::Duration = std::time::Duration::from_millis(16);
    event_loop.run(move |_event, _, control_flow| {
        let now = std::time::Instant::now();
        let mut delta = now - last_loop;
        if delta < FRAME_TIME_MIN {
            std::thread::sleep(FRAME_TIME_MIN - delta);
            delta = FRAME_TIME_MIN;
        }
        let delta = delta.as_secs_f32();
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

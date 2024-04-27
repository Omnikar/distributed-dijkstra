mod agent;
mod math;
mod sim;

use pixels::{PixelsBuilder, SurfaceTexture};
use rand::Rng;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::sim::render::Renderable;

const SCREEN_DIMS: (u32, u32) = (1440, 900);

fn main() {
    let mut rng = rand::thread_rng();

    let world_f = std::env::args().nth(1);
    let world_f = world_f.as_deref().unwrap_or("scenes/default.ron");
    let world_s = std::fs::read_to_string(world_f).unwrap();
    let mut world: sim::World = ron::from_str(&world_s).unwrap();
    world
        .obstacles
        .push(Box::new(sim::obstacle::InvRect(sim::obstacle::Rect {
            ranges: [0.0..world.world_size.x, 0.0..world.world_size.y],
        })));
    world.agents = sim::World::new().agents;
    for agent in &mut world.agents {
        if world
            .obstacles
            .iter()
            .map(|obs| (obs.bounding_box(), obs))
            .any(|(bbox, obs)| {
                [0, 1].map(|i| bbox[i].contains(&agent.pos[i])) == [true; 2]
                    && obs.inside(agent.pos)
            })
        {
            agent.pos = world.world_size / 2.0;
        }
    }

    let n_sites =
        world
            .sites
            .iter()
            .map(|site| site.kind + 1)
            .max()
            .unwrap_or(0);
    world.agents.iter_mut().for_each(|agent| {
        agent.state.sites = vec![(f32::INFINITY, true); n_sites];
        agent.state.sites[rng.gen_range(0..=1)].1 = false;
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

    let mut shortest_dist = f32::MAX;

    let trails = var("TRAILS");

    let mut trail_buf = vec![0u8; (SCREEN_DIMS.0 * SCREEN_DIMS.1 * 4) as usize].into_boxed_slice();

    let start = Instant::now();
    let mut last_loop = Instant::now();
    const FRAME_TIME_MIN: std::time::Duration = std::time::Duration::from_millis(16);
    event_loop.run(move |_event, _, control_flow| {
        let now = Instant::now();
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

        // world.agents[0].render(&mut sim::render::RenderArgs {
        //     world: &world,
        //     frame: &mut *trail_buf,
        //     px_per_unit: 90.0,
        //     px_width: 1440,
        // });

        if trails {
            for agent in &world.agents {
                if agent.is_scout {
                    continue;
                }
                agent.render(&mut sim::render::RenderArgs {
                    world: &world,
                    frame: &mut trail_buf,
                    px_per_unit: 90.0,
                    px_width: 1440,
                });
            }
            frame
                .iter_mut()
                .zip(trail_buf.iter())
                .for_each(|(v, &tr)| *v = tr.max(*v));
        }

        pixels.render().unwrap();

        if trails {
            trail_buf
                .iter_mut()
                .for_each(|v| *v = v.saturating_sub(rng.gen_bool(0.5) as u8));
        }

        let new_shortest_dist = world
            .agents
            .iter()
            .map(|a| a.shortest_dist)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();
        if new_shortest_dist < shortest_dist {
            shortest_dist = new_shortest_dist;
            println!("{}\t{shortest_dist}", start.elapsed().as_secs_f32());
        }

        if let ControlFlow::ExitWithCode(code) = control_flow {
            std::process::exit(*code);
        }
    });
}

fn var<T: std::str::FromStr + Default>(name: &'static str) -> T {
    // T::from_str(&std::env::var(name).unwrap())
    //     .ok()
    //     .unwrap_or_default()
    std::env::var(name)
        .ok()
        .and_then(|v| T::from_str(&v).ok())
        .unwrap_or_default()
}

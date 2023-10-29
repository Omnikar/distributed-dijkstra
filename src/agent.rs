use crate::{math::Vec2, sim::obstacle::Obstacle};

use rand::Rng;
use std::f32::consts::PI;

pub struct Agent {
    pub pos: Vec2,
    /// radians right-handedly counterclockwise from +x
    pub dir: f32,
    pub state: State,
    pub speed: f32,
    /// maximum turn speed
    pub turn: f32,
    /// communication distance
    pub comm: f32,
    pub is_scout: bool,
}

#[derive(Clone)]
pub struct State {
    /// site kind indexed, (square distance, is targeting site)
    pub sites: Vec<(f32, bool)>,
    /// currently targeted site (kind, distance)
    pub target: Option<(usize, f32)>,
}

#[derive(Clone, Copy)]
pub struct Message {
    pub site_kind: usize,
    pub sq_dist: f32,
    pub range: f32,
    pub source: Vec2,
}

impl Agent {
    pub fn step<'a>(
        &mut self,
        delta: f32,
        obstacles: impl Iterator<Item = &'a dyn Obstacle> + Clone,
    ) {
        // self.pos += self.speed * delta * Vec2::new(self.dir.cos(), self.dir.sin());
        let mut origin = self.pos;
        let mut pos_delta = self.speed * delta * Vec2::new(self.dir.cos(), self.dir.sin());
        let mut collision_limit = 0..10;
        while let Some((hit_pos, refl_delta)) = collision_limit.next().and_then(|_| {
            obstacles
                .clone()
                .find_map(|obs| obs.process_collision(origin, pos_delta))
        }) {
            origin = hit_pos;
            pos_delta = refl_delta;
        }
        self.pos = origin + pos_delta;
        self.dir = pos_delta.angle();

        self.state
            .sites
            .iter_mut()
            .for_each(|(sq_dist, _)| *sq_dist = (sq_dist.sqrt() + self.speed * delta).powi(2));

        self.dir += rand::thread_rng().gen_range(-delta * self.turn..delta * self.turn);
        self.dir = self.dir.rem_euclid(2.0 * PI);
    }

    pub fn inform(&mut self, msg: Message) -> Option<Message> {
        let state = self
            .state
            .sites
            .get_mut(msg.site_kind)
            .filter(|st| msg.sq_dist < st.0)?;

        state.0 = msg.sq_dist;

        if state.1 && !self.is_scout {
            if self
                .state
                .target
                .map(|site| msg.sq_dist <= site.1)
                .unwrap_or(true)
            {
                self.state.target = Some((msg.site_kind, state.0));
                // let diff = [0, 1].map(|i| msg.source[i] - self.pos[i]);
                let diff = msg.source - self.pos;
                self.dir = diff.y.atan2(diff.x);
            }

            if msg.sq_dist == 0.0 {
                self.dir = rand::thread_rng().gen_range(0.0..2.0 * PI);
                state.1 = false;

                if self.state.sites.iter().all(|site| !site.1) {
                    self.state.sites.iter_mut().for_each(|site| site.1 = true);
                }

                self.state.target = None;
            }
        }

        Some(Message {
            sq_dist: (msg.sq_dist.sqrt() + self.comm).powi(2),
            range: self.comm,
            source: self.pos,
            ..msg
        })
    }

    pub fn contain(&mut self, world_size: Vec2) {
        use std::f32::consts::FRAC_PI_2;
        [0, 1].map(|i| {
            // <0: outside negative
            //  0: inside
            // >0: outside positive
            let pos_status = (self.pos[i] / world_size[i]).floor() as i32;
            // <0: facing outside negative
            // >0: facing outside positive
            let dir_status = !(FRAC_PI_2..3.0 * FRAC_PI_2)
                .contains(&(self.dir - i as f32 * FRAC_PI_2)) as i32
                * 2
                - 1;
            if pos_status * dir_status > 0 {
                self.dir = ((1 - i) as f32 * PI - self.dir).rem_euclid(2.0 * PI);
            }
        });
    }
}

impl crate::sim::render::Renderable for Agent {
    fn render(&self, args: crate::sim::render::Args) {
        let px_coord = (self.pos * args.px_per_unit).map(|v| v as usize);

        let color = if self.is_scout {
            [0x4e; 3]
        } else {
            self.state
                .target
                .map(|site| args.world.site_kinds[site.0])
                .unwrap_or([0xff; 3])
        };

        crate::sim::render::put_px(args, px_coord, color);
    }
}

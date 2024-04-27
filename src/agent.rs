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
    /// obstacle avoidance distance
    pub obs_dist: f32,
    pub is_scout: bool,
    pub shortest_dist: f32,
    pub current_dist: f32,
}

#[derive(Clone)]
pub struct State {
    /// site kind indexed, (square distance, is targeting site)
    pub sites: Vec<(f32, bool)>,
    /// currently targeted site (kind, square distance)
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
        // let speed = if !self.is_scout && self.state.target.is_none() {
        //     self.speed * 0.2
        // } else {
        //     self.speed
        // };
        // let speed = self.speed;
        let speed = if self.is_scout {
            self.speed * 1.5
        } else {
            self.speed
        };

        // self.pos += self.speed * delta * Vec2::new(self.dir.cos(), self.dir.sin());
        let mut origin = self.pos;
        let dist = speed * delta;
        self.current_dist += dist;
        let mut pos_delta = dist * Vec2::new(self.dir.cos(), self.dir.sin());
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

        for (kind, sq_dist) in self.state.sites.iter_mut().map(|(d, _)| d).enumerate() {
            *sq_dist = (sq_dist.sqrt() + speed * delta).powi(2);
            if let Some(t_dist) = self
                .state
                .target
                .as_mut()
                .and_then(|(t_kind, t_dist)| (*t_kind == kind).then_some(t_dist))
            {
                *t_dist = *sq_dist;
            }
        }

        self.dir += rand::thread_rng().gen_range(-delta * self.turn..delta * self.turn);
        self.dir = self.dir.rem_euclid(2.0 * PI);
    }

    pub fn inform(&mut self, msg: Message) -> Option<Message> {
        // msg.sq_dist = msg.sq (msg.source - self.pos).mag();
        // msg.sq_dist = (msg.sq_dist.sqrt() - msg.range + (msg.source - self.pos).mag())
        //     .powi(2)
        //     .min(msg.sq_dist);
        let state = self
            .state
            .sites
            .get_mut(msg.site_kind)
            .filter(|st| msg.sq_dist < st.0)?;
        // else {
        //     return Vec::new();
        // }

        state.0 = msg.sq_dist;

        if state.1 && !self.is_scout {
            if self
                .state
                .target
                .map(|site| msg.sq_dist < site.1)
                .unwrap_or(true)
            {
                self.state.target = Some((msg.site_kind, state.0));
                let diff = msg.source - self.pos;
                self.dir = diff.y.atan2(diff.x);
            }
            // 180 away from messages about non targeted sites
            // else {
            //     let diff = self.pos - msg.source;
            //     self.dir = diff.y.atan2(diff.x);
            // }

            if msg.sq_dist == 0.0 {
                // self.dir = rand::thread_rng().gen_range(0.0..2.0 * PI);
                self.dir = (self.dir + PI).rem_euclid(2.0 * PI);
                state.1 = false;

                if self.state.sites.iter().all(|site| !site.1) {
                    self.state
                        .sites
                        .iter_mut()
                        .enumerate()
                        .for_each(|(i, site)| site.1 = i != msg.site_kind);
                }

                if self.current_dist.is_nan() {
                    self.current_dist = 0.0;
                } else if self
                    .state
                    .target
                    .map(|v| v.0 == msg.site_kind)
                    .unwrap_or(false)
                {
                    self.shortest_dist = self.shortest_dist.min(self.current_dist);
                    self.current_dist = 0.0;
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
        // vec![
        //     // Message {
        //     //     sq_dist: (msg.sq_dist.sqrt() + self.comm).powi(2),
        //     //     range: self.comm,
        //     //     source: self.pos,
        //     //     ..msg
        //     // },
        //     // Message {
        //     //     sq_dist: (msg.sq_dist.sqrt() + 3.0 * self.comm).powi(2),
        //     //     range: self.comm,
        //     //     source: self.pos,
        //     //     ..msg
        //     // },
        //     Message {
        //         sq_dist: (msg.sq_dist.sqrt() + 9.0 * self.comm).powi(2),
        //         range: self.comm,
        //         source: self.pos,
        //         ..msg
        //     },
        // ]
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

    pub fn avoid_obstacles<'a>(&mut self, obstacles: impl Iterator<Item = &'a dyn Obstacle>) {
        let ray = Vec2::new(self.dir.cos(), self.dir.sin());
        let Some((t, norm)) = obstacles
            .flat_map(|o| o.intersects(self.pos, ray))
            .filter(|&(t, _)| t > 0.0 && t < self.obs_dist)
            .min_by(|a, b| a.0.total_cmp(&b.0))
        else {
            return;
        };
        let t = t - 0.1;

        use std::f32::consts::FRAC_PI_2;

        let tan = Vec2::new(norm.y, -norm.x);
        let tan_comp = ray.dot(tan).acos() - FRAC_PI_2;
        let norm_comps = [-1.0, 1.0].map(|sign| (t * sign * ray.dot(norm) / self.obs_dist).acos());
        let angles = norm_comps
            .map(|norm_comp| tan_comp + norm_comp)
            .map(|v| (v + FRAC_PI_2).rem_euclid(PI) - FRAC_PI_2);
        let mut angle = angles[(angles[0].abs() > angles[1].abs()) as usize];
        angle += 10f32.to_radians().copysign(angle);
        self.dir += angle;
    }
}

impl crate::sim::render::Renderable for Agent {
    fn render(&self, args: crate::sim::render::Args) {
        let px_coord = (self.pos * args.px_per_unit).map(|v| v as usize);

        let trails = crate::var("TRAILS");

        let color = if self.is_scout {
            // [0x4e; 3]
            if trails {
                [0x10; 3]
            } else {
                [0x4e; 3]
            }
        } else if trails {
            self.state
                .target
                .map(|site| args.world.site_kinds[site.0].map(|v| (v as u16 * 0x30 / 0xff) as u8))
                .unwrap_or([0x10; 3])
        } else {
            self.state
                .target
                .map(|site| args.world.site_kinds[site.0])
                .unwrap_or([0xff; 3])
        };

        if trails {
            crate::sim::render::add_px(args, px_coord, color);
        } else {
            crate::sim::render::put_px(args, px_coord, color);
        }
    }
}

use rand::Rng;
use std::f32::consts::PI;

pub struct Agent {
    pub pos: [f32; 2],
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
    pub source: [f32; 2],
}

impl Agent {
    pub fn step(&mut self, delta: f32) {
        self.pos[0] += self.speed * delta * self.dir.cos();
        self.pos[1] += self.speed * delta * self.dir.sin();

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
                let diff = [0, 1].map(|i| msg.source[i] - self.pos[i]);
                self.dir = diff[1].atan2(diff[0]);
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

    pub fn contain(&mut self, world_size: [f32; 2]) {
        use std::f32::consts::FRAC_PI_2;
        [0, 1].map(|i| {
            // <0: outside negative
            //  0: inside
            // >0: outside positive
            let pos_status = (self.pos[i] / world_size[i]).floor() as i32;
            // <0: facing outside negative
            // >0: facing outside positive
            let dir_status = (self.dir - i as f32 * FRAC_PI_2 < FRAC_PI_2) as i32 * 2 - 1;
            if pos_status * dir_status > 0 {
                self.dir = (1 - i) as f32 * PI - self.dir;
            }
        });
    }
}

impl crate::sim::Renderable for Agent {
    fn render(
        &self,
        world: &crate::sim::World,
        frame: &mut [u8],
        px_per_unit: f32,
        px_width: usize,
    ) {
        let px_coord = self.pos.map(|coord| (coord * px_per_unit) as usize);

        let idx = 4 * (px_coord[1] * px_width + px_coord[0]);
        if idx >= frame.len() {
            return;
        }

        let color = if self.is_scout {
            [0x4e; 3]
        } else {
            self.state
                .target
                .map(|site| world.site_kinds[site.0])
                .unwrap_or([0xff; 3])
        };

        frame[idx..idx + 3].copy_from_slice(&color);
    }
}

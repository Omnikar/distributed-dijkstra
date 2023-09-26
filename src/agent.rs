use rand::Rng;
use std::collections::HashMap;
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
}

#[derive(Clone)]
pub struct State {
    /// site kind -> (square distance, is targeting site)
    pub sites: HashMap<usize, (f32, bool)>,
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
            .for_each(|(_, (sq_dist, _))| *sq_dist = (sq_dist.sqrt() + self.speed * delta).powi(2));

        self.dir += rand::thread_rng().gen_range(-delta * self.turn..delta * self.turn);
        self.dir = self.dir.rem_euclid(2.0 * PI);
    }

    pub fn inform(&mut self, msg: Message) -> Option<Message> {
        let state = self
            .state
            .sites
            .get_mut(&msg.site_kind)
            .filter(|st| msg.sq_dist < st.0)?;

        state.0 = msg.sq_dist;

        if state.1 {
            self.state.target = Some((msg.site_kind, state.0));

            let diff = [0, 1].map(|i| msg.source[i] - self.pos[i]);
            self.dir = diff[1].atan2(diff[0]);

            if msg.sq_dist == 0.0 {
                self.dir = rand::thread_rng().gen_range(0.0..2.0 * PI);
                state.1 = false;
                self.state
                    .sites
                    .get_mut(&((msg.site_kind + 1) % 2))
                    .unwrap()
                    .1 = true;

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
        [0, 1].map(|i| {
            // <0: outside negative
            //  0: inside
            // >0: outside positive
            if (self.pos[i] / world_size[i]).floor() != 0.0 {
                self.dir = (1 - i) as f32 * PI - self.dir;
            }
        });
    }
}

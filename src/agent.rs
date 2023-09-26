use rand::Rng;
use std::collections::HashMap;

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

        self.dir += rand::thread_rng().gen_range(-delta * self.turn..delta * self.turn);
    }

    pub fn inform(&mut self, msg: Message) -> Option<Message> {
        let state = self
            .state
            .sites
            .get_mut(&msg.site_kind)
            .filter(|st| msg.sq_dist < st.0)?;

        if msg.sq_dist == 0.0 {
            state.1 = false;
        }

        state.0 = msg.sq_dist;
        if state.1 {
            let diff = [0, 1].map(|i| msg.source[i] - self.pos[i]);
            self.dir = diff[1].atan2(diff[0]);
        }

        Some(Message {
            sq_dist: (msg.sq_dist.sqrt() + self.comm).powi(2),
            range: self.comm,
            source: self.pos,
            ..msg
        })
    }
}

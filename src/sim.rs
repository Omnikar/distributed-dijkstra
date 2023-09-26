use crate::agent::{Agent, Message};

use rand::Rng;

pub struct World {
    pub agents: Vec<Agent>,
    pub sites: Vec<Site>,
    pub site_kinds: Vec<[u8; 3]>,
    pub world_size: [f32; 2],
    msg_queue: std::collections::VecDeque<Message>,
}

pub struct Site {
    pub pos: [f32; 2],
    pub kind: usize,
    pub size: f32,
}

impl World {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let empty_state = crate::agent::State {
            sites: Default::default(),
            target: None,
        };
        let agents = (0..2000).map(|_| {
            let x_coord = rng.gen_range(7.5..=8.5);
            let y_coord = rng.gen_range(4.5..=5.5);
            Agent {
                pos: [x_coord, y_coord],
                dir: rng.gen_range(0.0..2.0 * std::f32::consts::PI),
                state: empty_state.clone(),
                speed: rng.gen_range(0.3..=1.3),
                turn: 100.0f32.to_radians(),
                comm: 0.8,
            }
        });

        Self {
            agents: agents.collect(),
            sites: Vec::new(),
            site_kinds: Vec::new(),
            world_size: [16.0, 10.0],
            msg_queue: Default::default(),
        }
    }

    pub fn render(&self, frame: &mut [u8], px_per_unit: f32, px_width: usize) {
        for site in &self.sites {
            let px = |v| (v * px_per_unit) as usize;

            let [x_range, y_range] = site
                .pos
                .map(|coord| px(coord - site.size)..=px(coord + site.size));
            let bbox_iter = x_range.flat_map(|x| y_range.clone().map(move |y| [x, y]));

            let site_px_pos = site.pos.map(px);
            let sq_size = px(site.size).pow(2);

            for coord in bbox_iter {
                let diff = [0, 1].map(|i| site_px_pos[i].abs_diff(coord[i]));

                let sq_dist: usize = diff.map(|x| x.pow(2)).into_iter().sum();

                if sq_dist <= sq_size {
                    let idx = 4 * (coord[1] * px_width + coord[0]);
                    frame[idx..idx + 3].copy_from_slice(&self.site_kinds[site.kind].map(|v| v / 2));
                }
            }
        }

        for (i, agent) in self.agents.iter().enumerate() {
            let px_coord = agent.pos.map(|coord| (coord * px_per_unit) as usize);

            let idx = 4 * (px_coord[1] * px_width + px_coord[0]);
            if idx >= frame.len() {
                continue;
            }

            let mut color = agent
                .state
                .target
                .map(|site| self.site_kinds[site.0])
                .unwrap_or([0xff; 3]);
            if i < self.agents.len() - 30 {
                // color = color.map(|v| v / 3 * 2);
            }

            frame[idx..idx + 3].copy_from_slice(&color);
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.agents.iter_mut().for_each(|agent| {
            agent.step(delta);
            agent.contain(self.world_size);
        });
        self.msg_queue
            .extend(self.sites.iter().map(Site::collision_msg));
        while let Some(msg) = self.msg_queue.pop_front() {
            self.process_message(msg);
        }
    }

    fn process_message(&mut self, msg: Message) {
        let sq_range = msg.range.powi(2);
        for agent in &mut self.agents {
            let abs_diff = [0, 1].map(|i| (agent.pos[i] - msg.source[i]).abs());
            // Outside bounding box
            if abs_diff.into_iter().any(|v| v > msg.range)
                // Outside circle
                || abs_diff.into_iter().map(|v| v * v).sum::<f32>() > sq_range
            {
                continue;
            }

            if let Some(new_msg) = agent.inform(msg) {
                self.msg_queue.push_back(new_msg);
            }
        }
    }
}

impl Site {
    fn collision_msg(&self) -> Message {
        Message {
            site_kind: self.kind,
            sq_dist: 0.0,
            range: self.size,
            source: self.pos,
        }
    }
}

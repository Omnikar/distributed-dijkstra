pub mod obstacle;
pub mod render;
pub mod site;

use crate::agent::{Agent, Message};
use crate::math::Vec2;
use obstacle::Obstacle;
use render::Renderable;
use site::Site;

use rand::Rng;

#[derive(serde::Deserialize)]
pub struct World {
    #[serde(skip)]
    pub agents: Vec<Agent>,
    pub sites: Vec<Site>,
    pub site_kinds: Vec<[u8; 3]>,
    #[serde(deserialize_with = "obstacle::deser_obstacles")]
    pub obstacles: Vec<Box<dyn Obstacle>>,
    pub world_size: Vec2,
    #[serde(skip)]
    msg_queue: std::collections::VecDeque<Message>,
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
                pos: (x_coord, y_coord).into(),
                dir: rng.gen_range(0.0..2.0 * std::f32::consts::PI),
                state: empty_state.clone(),
                speed: rng.gen_range(0.3..=1.3),
                turn: 100.0f32.to_radians(),
                comm: 0.8,
                is_scout: rng.gen_bool(0.4),
            }
        });

        Self {
            agents: agents.collect(),
            sites: Vec::new(),
            site_kinds: Vec::new(),
            obstacles: Vec::new(),
            world_size: (16.0, 10.0).into(),
            msg_queue: Default::default(),
        }
    }

    pub fn render(&self, frame: &mut [u8], px_per_unit: f32, px_width: usize) {
        let mut args = render::RenderArgs {
            world: self,
            frame,
            px_per_unit,
            px_width,
        };

        for obstacle in &self.obstacles {
            obstacle.render(&mut args);
        }

        for site in &self.sites {
            site.render(&mut args);
        }

        for agent in &self.agents {
            agent.render(&mut args);
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.agents.iter_mut().for_each(|agent| {
            agent.step(delta, self.obstacles.iter().map(Box::as_ref));
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
                // Outside circle; only checked if inside bounding box
                || abs_diff.into_iter().map(|v| v * v).sum::<f32>() > sq_range
            {
                continue;
            }

            if self
                .obstacles
                .iter()
                .flat_map(|obs| obs.intersects(msg.source, agent.pos - msg.source))
                .any(|(t, _)| 0.0 < t && t <= 1.0)
            {
                continue;
            }

            if let Some(new_msg) = agent.inform(msg) {
                self.msg_queue.push_back(new_msg);
            }
        }
    }
}

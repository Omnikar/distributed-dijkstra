use crate::{agent::Message, math::Vec2};

#[derive(serde::Deserialize)]
pub struct Site {
    pub pos: Vec2,
    pub kind: usize,
    pub size: f32,
}

impl Site {
    pub fn collision_msg(&self) -> Message {
        Message {
            site_kind: self.kind,
            sq_dist: 0.0,
            range: self.size,
            source: self.pos,
        }
    }
}

impl super::render::Renderable for Site {
    fn render(&self, args: super::render::Args) {
        super::render::draw_circle(
            args,
            self.pos,
            self.size,
            args.world.site_kinds[self.kind].map(|v| v / 2),
        );
    }
}

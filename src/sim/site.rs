use crate::{agent::Message, math::Vec2};

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

impl super::Renderable for Site {
    fn render(&self, world: &super::World, frame: &mut [u8], px_per_unit: f32, px_width: usize) {
        let px = |v| (v * px_per_unit) as usize;

        let [x_range, y_range] = self
            .pos
            .map(|coord| px(coord - self.size)..=px(coord + self.size));
        let bbox_iter = x_range.flat_map(|x| y_range.clone().map(move |y| [x, y]));

        let site_px_pos = self.pos.map(px);
        let sq_size = px(self.size).pow(2);

        for coord in bbox_iter {
            let diff = [0, 1].map(|i| site_px_pos[i].abs_diff(coord[i]));

            let sq_dist: usize = diff.map(|x| x.pow(2)).into_iter().sum();

            if sq_dist <= sq_size {
                let idx = 4 * (coord[1] * px_width + coord[0]);
                frame[idx..idx + 3].copy_from_slice(&world.site_kinds[self.kind].map(|v| v / 2));
            }
        }
    }
}

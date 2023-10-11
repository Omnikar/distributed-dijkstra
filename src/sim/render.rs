use crate::math::Vec2;

use super::World;

pub type Args<'a, 'b, 'c> = &'a mut RenderArgs<'b, 'c>;

pub struct RenderArgs<'a, 'b> {
    pub world: &'a World,
    pub frame: &'b mut [u8],
    pub px_per_unit: f32,
    pub px_width: usize,
}

pub trait Renderable {
    // fn render(&self, world: &World, frame: &mut [u8], px_per_unit: f32, px_width: usize);
    fn render(&self, args: Args);
}

pub fn put_px(args: Args, px_coord: [usize; 2], color: [u8; 3]) {
    let idx = 4 * (px_coord[1] * args.px_width + px_coord[0]);
    if idx >= args.frame.len() {
        return;
    }

    args.frame[idx..idx + 3].copy_from_slice(&color);
}

pub fn draw_circle(args: Args, center: Vec2, radius: f32, color: [u8; 3]) {
    let px = |v| (v * args.px_per_unit) as usize;

    let [x_range, y_range] = center.map(|coord| px(coord - radius)..=px(coord + radius));
    let bbox_iter = x_range.flat_map(|x| y_range.clone().map(move |y| [x, y]));

    let site_px_pos = center.map(px);
    let sq_size = px(radius).pow(2);

    for coord in bbox_iter {
        let diff = [0, 1].map(|i| site_px_pos[i].abs_diff(coord[i]));

        let sq_dist: usize = diff.map(|x| x.pow(2)).into_iter().sum();

        if sq_dist <= sq_size {
            let idx = 4 * (coord[1] * args.px_width + coord[0]);
            args.frame[idx..idx + 3].copy_from_slice(&color);
        }
    }
}

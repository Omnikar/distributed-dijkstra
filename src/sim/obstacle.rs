use serde::Deserialize;
use std::ops::Range;

use super::render::Renderable;
use crate::math::Vec2;

pub trait Obstacle {
    fn bounding_box(&self) -> [Range<f32>; 2];
    /// Assuming a point is *inside the bounding box*, is it in the shape?
    fn inside(&self, coord: Vec2) -> bool;
    /// -> (ray multiplier, normal vector)
    fn intersects(&self, origin: Vec2, ray: Vec2) -> Vec<(f32, Vec2)>;

    /// -> (hit pos, delta)
    fn process_collision(&self, origin: Vec2, delta: Vec2) -> Option<(Vec2, Vec2)> {
        let bbox = self.bounding_box();
        let end_pos = origin + delta;
        if !([0, 1].map(|i| bbox[i].contains(&end_pos[i])) == [true; 2] && self.inside(end_pos)) {
            return None;
        }

        // eprintln!("collision");

        let hits = self.intersects(origin, delta);
        let Some((t, norm)) = hits
            .into_iter()
            .filter(|&(t, _)| -1.0 < t && t <= 1.0)
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
        else {
            // eprintln!("oops");
            return None;
        };

        let hit_pos = origin + t * delta;
        let rest_delta = (1.0 - t) * delta;
        let refl_delta = rest_delta - 2.0 * norm * rest_delta.dot(norm);

        Some((hit_pos, refl_delta))
    }
}

impl Renderable for Box<dyn Obstacle> {
    fn render(&self, args: super::render::Args) {
        let px_per_unit = args.px_per_unit;
        let px = |v| (v * px_per_unit) as usize;
        let unpx = |v| v as f32 / px_per_unit;

        let bbox = self.bounding_box();
        // let [x_range, y_range] = self.bounding_box().map(|r| px(r.start)..=px(r.end));
        let [x_range, y_range] = [0, 1]
            .map(|i| px(bbox[i].start.max(0.0))..=px(bbox[i].end.min(args.world.world_size[i])));
        let bbox_iter = x_range.flat_map(|x| y_range.clone().map(move |y| [x, y]));

        for px_coord in bbox_iter {
            if self.inside(px_coord.map(unpx).into()) {
                super::render::put_px(args, px_coord, [0x85, 0x53, 0x09]);
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Obstacle for Circle {
    fn bounding_box(&self) -> [Range<f32>; 2] {
        let rad_vec = Vec2::new(self.radius, self.radius);
        let bot_left = self.center - rad_vec;
        let top_right = self.center + rad_vec;
        // [bot_left.x..=top_right.x, bot_left.y..=top_right.y]
        [0, 1].map(|i| bot_left[i]..top_right[i])
    }

    fn inside(&self, coord: Vec2) -> bool {
        // let diff = coord - self.center;
        // diff.map(|coord| coord.abs() < self.radius) == [true; 2]
        //     && diff.sq_mag() < self.radius.powi(2)
        (coord - self.center).sq_mag() < self.radius.powi(2)
    }

    fn intersects(&self, origin: Vec2, ray: Vec2) -> Vec<(f32, Vec2)> {
        let sq_r = self.radius.powi(2);
        let diff = self.center - origin;
        let diff_sq_mag = diff.sq_mag();
        let v_sq_mag = ray.sq_mag();
        let v_dot_diff = ray.dot(diff);

        let const_term = v_dot_diff / v_sq_mag;
        let pm_term = (v_dot_diff.powi(2) - v_sq_mag * (diff_sq_mag - sq_r)).sqrt() / v_sq_mag;

        let ts = [-1.0, 1.0].map(|v| const_term + v * pm_term);
        let norms = ts
            .map(|t| origin + t * ray)
            .map(|hit| (hit - self.center).norm());

        [0, 1].map(|i| (ts[i], norms[i])).into()
    }
}

// Vertices must be supplied in right-handedly counterclockwise order.
#[derive(Deserialize)]
#[serde(from = "[Vec2; 3]")]
pub struct Triangle {
    pub verts: [Vec2; 3],
}

impl From<[Vec2; 3]> for Triangle {
    fn from(verts: [Vec2; 3]) -> Self {
        Self { verts }
    }
}

impl Obstacle for Triangle {
    fn bounding_box(&self) -> [Range<f32>; 2] {
        let xs = self.verts.map(|vert| vert.x);
        let ys = self.verts.map(|vert| vert.y);

        [xs, ys].map(|vs| {
            vs.into_iter().min_by(f32::total_cmp).unwrap()
                ..vs.into_iter().max_by(f32::total_cmp).unwrap()
        })
    }

    fn inside(&self, coord: Vec2) -> bool {
        let abv_line = |right: Vec2, left: Vec2| -> bool {
            (right - coord).cross(left - coord).is_sign_negative()
        };

        let mut shifted_verts = self.verts;
        shifted_verts.rotate_right(1);
        self.verts
            .into_iter()
            .zip(shifted_verts)
            .all(|(right, left)| abv_line(right, left))
    }

    fn intersects(&self, origin: Vec2, ray: Vec2) -> Vec<(f32, Vec2)> {
        let line_int = |p1: Vec2, p2: Vec2| -> Option<(f32, Vec2)> {
            let diff = p2 - p1;
            let coefs = Vec2::new(-diff.y, diff.x);
            Some(-(coefs.dot(origin) + p1.cross(p2)) / coefs.dot(ray))
                .filter(|&t| {
                    let end = origin + t * ray;
                    (p1 - end).dot(p2 - end).is_sign_negative()
                })
                .map(|t| (t, -coefs.norm()))
        };

        let mut shifted_verts = self.verts;
        shifted_verts.rotate_right(1);
        self.verts
            .into_iter()
            .zip(shifted_verts)
            .filter_map(|(p2, p1)| line_int(p1, p2))
            .collect()
    }
}

// Corners must be supplied as (x min, y min), (x max, y max)
#[derive(Deserialize)]
#[serde(from = "[Range<f32>; 2]")]
pub struct Rect {
    pub ranges: [Range<f32>; 2],
}

impl From<[Range<f32>; 2]> for Rect {
    fn from(ranges: [Range<f32>; 2]) -> Self {
        Self { ranges }
    }
}

impl Obstacle for Rect {
    fn bounding_box(&self) -> [Range<f32>; 2] {
        self.ranges.clone()
    }

    fn inside(&self, _coord: Vec2) -> bool {
        true
    }

    fn intersects(&self, origin: Vec2, ray: Vec2) -> Vec<(f32, Vec2)> {
        [0, 1]
            .into_iter()
            .flat_map(|axis| {
                [0, 1]
                    .map(|extr| {
                        let mut norm = [extr as f32 * 2.0 - 1.0, 0.0];
                        norm.rotate_right(axis);
                        let range = &self.ranges[axis];
                        (
                            ([range.start, range.end][extr] - origin[axis]) / ray[axis],
                            norm.into(),
                        )
                    })
                    .into_iter()
                    .filter(move |&(t, _)| {
                        self.ranges[1 - axis].contains(&(origin + t * ray)[1 - axis])
                    })
            })
            .collect()
    }
}

pub struct InvRect(pub Rect);
impl Obstacle for InvRect {
    fn bounding_box(&self) -> [Range<f32>; 2] {
        [f32::MIN..f32::MAX, f32::MIN..f32::MAX]
    }

    fn inside(&self, coord: Vec2) -> bool {
        let bbox = self.0.bounding_box();
        [0, 1].map(|i| bbox[i].contains(&coord[i])) != [true, true]
    }

    fn intersects(&self, origin: Vec2, ray: Vec2) -> Vec<(f32, Vec2)> {
        let mut ints = self.0.intersects(origin, ray);
        ints.iter_mut().for_each(|(_, norm)| *norm *= -1.0);
        ints
    }
}

pub fn deser_obstacles<'de, D>(d: D) -> Result<Vec<Box<dyn Obstacle>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    macro_rules! gen_obs_deser {
            ($($obs:ident),*) => {
                #[derive(Deserialize)]
                #[serde(untagged)]
                enum ObsObj { $($obs($obs)),* }

                let obj_vec: Vec<ObsObj> = Deserialize::deserialize(d)?;

                return Ok(obj_vec
                    .into_iter()
                    .map(|o| match o {
                        $(
                            ObsObj::$obs(v) => Box::new(v) as Box<dyn Obstacle>,
                        )*
                    })
                    .collect());
            };
        }

    gen_obs_deser!(Circle, Triangle, Rect);
}

#![allow(dead_code)]

use std::f64;

use raft_zero::NodeId;

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    scaled_cos: f64,
    scaled_sin: f64,
    offset: [f64; 2],
}

impl Transform {
    pub fn from_segment(a: [f64; 2], b: [f64; 2]) -> Self {
        let diff = vec_sub(b, a);
        Self {
            scaled_cos: diff[0],
            scaled_sin: diff[1],
            offset: a,
        }
    }
    pub fn xform_point(self, a: [f64; 2]) -> [f64; 2] {
        [
            a[0] * self.scaled_cos - a[1] * self.scaled_sin + self.offset[0],
            a[0] * self.scaled_sin + a[1] * self.scaled_cos + self.offset[1],
        ]
    }
    pub fn scale(self) -> f64 {
        vec_len([self.scaled_cos, self.scaled_sin])
    }
    pub fn rescale(&mut self) -> f64 {
        let scale = self.scale();
        self.scaled_cos /= scale;
        self.scaled_sin /= scale;
        scale
    }
}

pub fn node_pos(node_id: NodeId, area_size: f64, num_nodes: usize) -> [f64; 2] {
    let angle = f64::consts::PI * 2.0 * node_id.0 as f64 / num_nodes as f64;
    [angle.sin() * area_size * 0.4, angle.cos() * area_size * 0.4]
}

pub fn vec_lerp(a: [f64; 2], b: [f64; 2], c: f64) -> [f64; 2] {
    let d = 1.0 - c;
    [a[0] * d + b[0] * c, a[1] * d + b[1] * c]
}

pub fn vec_add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

pub fn vec_mul(a: [f64; 2], b: f64) -> [f64; 2] {
    [a[0] * b, a[1] * b]
}

pub fn vec_sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

pub fn vec_perp(a: [f64; 2]) -> [f64; 2] {
    [-a[1], a[0]]
}

pub fn vec_len_sq(a: [f64; 2]) -> f64 {
    a[0] * a[0] + a[1] * a[1]
}

pub fn vec_len(a: [f64; 2]) -> f64 {
    vec_len_sq(a).sqrt()
}

pub fn vec_dist_sq(a: [f64; 2], b: [f64; 2]) -> f64 {
    vec_len_sq(vec_sub(b, a))
}

pub fn vec_dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    vec_len(vec_sub(b, a))
}

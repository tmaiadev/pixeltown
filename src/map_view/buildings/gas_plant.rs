use macroquad::prelude::*;

use super::{Block, Model, Stack};

const fn sprite(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

const TURBINE_HALL: Block = Block {
    base: sprite(4.0, 4.0, 52.0, 24.0),
    height: 14.0,
    roof: sprite(100.0, 192.0, 52.0, 24.0),
    wall_ns: sprite(100.0, 220.0, 52.0, 14.0),
    wall_ew: sprite(156.0, 220.0, 24.0, 14.0),
};

const HEAT_RECOVERY: Block = Block {
    base: sprite(58.0, 4.0, 18.0, 24.0),
    height: 18.0,
    roof: sprite(156.0, 192.0, 18.0, 24.0),
    wall_ns: sprite(100.0, 238.0, 18.0, 18.0),
    wall_ew: sprite(122.0, 238.0, 24.0, 18.0),
};

const CONTROL_ROOM: Block = Block {
    base: sprite(4.0, 32.0, 20.0, 12.0),
    height: 8.0,
    roof: sprite(88.0, 128.0, 20.0, 12.0),
    wall_ns: sprite(136.0, 116.0, 20.0, 8.0),
    wall_ew: sprite(160.0, 116.0, 12.0, 8.0),
};

const BODY: Rect = sprite(184.0, 192.0, 10.0, 24.0);
const CAP: Rect = sprite(196.0, 192.0, 10.0, 10.0);

const SPHERE_BODY: Rect = sprite(210.0, 192.0, 16.0, 8.0);
const SPHERE_CAP: Rect = sprite(230.0, 192.0, 16.0, 16.0);

const fn stack(x: f32, y: f32) -> Stack {
    Stack { base: Vec2::new(x, y), height: 26.0, body: BODY, cap: CAP, plume: false }
}

const fn sphere(x: f32) -> Stack {
    Stack { base: Vec2::new(x, 46.0), height: 8.0, body: SPHERE_BODY, cap: SPHERE_CAP, plume: false }
}

pub const MODEL: Model = Model {
    yard: sprite(0.0, 192.0, 96.0, 64.0),
    blocks: &[TURBINE_HALL, HEAT_RECOVERY, CONTROL_ROOM],
    stacks: &[stack(88.0, 10.0), stack(88.0, 22.0), sphere(36.0), sphere(58.0), sphere(80.0)],
};

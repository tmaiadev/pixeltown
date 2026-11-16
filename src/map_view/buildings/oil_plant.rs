use macroquad::prelude::*;

use super::{Block, Model, Stack};

const fn sprite(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

const MACHINE_HALL: Block = Block {
    base: sprite(4.0, 4.0, 44.0, 28.0),
    height: 16.0,
    roof: sprite(88.0, 96.0, 44.0, 28.0),
    wall_ns: sprite(136.0, 96.0, 44.0, 16.0),
    wall_ew: sprite(184.0, 96.0, 28.0, 16.0),
};

const CONTROL_ROOM: Block = Block {
    base: sprite(4.0, 36.0, 20.0, 12.0),
    height: 8.0,
    roof: sprite(88.0, 128.0, 20.0, 12.0),
    wall_ns: sprite(136.0, 116.0, 20.0, 8.0),
    wall_ew: sprite(160.0, 116.0, 12.0, 8.0),
};

const BODY: Rect = sprite(214.0, 96.0, 8.0, 40.0);
const CAP: Rect = sprite(224.0, 96.0, 8.0, 8.0);

const BIG_TANK_BODY: Rect = sprite(232.0, 96.0, 18.0, 14.0);
const BIG_TANK_CAP: Rect = sprite(254.0, 96.0, 18.0, 18.0);
const SMALL_TANK_BODY: Rect = sprite(232.0, 140.0, 14.0, 12.0);
const SMALL_TANK_CAP: Rect = sprite(248.0, 140.0, 14.0, 14.0);

const fn tank(x: f32, y: f32, body: Rect, cap: Rect, height: f32) -> Stack {
    Stack { base: Vec2::new(x, y), height, body, cap, plume: false }
}

pub const MODEL: Model = Model {
    yard: sprite(0.0, 96.0, 80.0, 96.0),
    blocks: &[MACHINE_HALL, CONTROL_ROOM],
    stacks: &[
        Stack { base: Vec2::new(62.0, 12.0), height: 40.0, body: BODY, cap: CAP, plume: true },
        tank(18.0, 62.0, BIG_TANK_BODY, BIG_TANK_CAP, 7.0),
        tank(42.0, 62.0, BIG_TANK_BODY, BIG_TANK_CAP, 7.0),
        tank(66.0, 62.0, SMALL_TANK_BODY, SMALL_TANK_CAP, 6.0),
        tank(18.0, 83.0, BIG_TANK_BODY, BIG_TANK_CAP, 7.0),
        tank(42.0, 83.0, BIG_TANK_BODY, BIG_TANK_CAP, 7.0),
        tank(66.0, 83.0, SMALL_TANK_BODY, SMALL_TANK_CAP, 6.0),
    ],
};

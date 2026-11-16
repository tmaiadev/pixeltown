use macroquad::prelude::*;

use super::{Block, Model, Stack};

const fn sprite(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect { x, y, w, h }
}

const BOILER_HOUSE: Block = Block {
    base: sprite(4.0, 4.0, 44.0, 30.0),
    height: 20.0,
    roof: sprite(80.0, 0.0, 44.0, 30.0),
    wall_ns: sprite(128.0, 0.0, 44.0, 20.0),
    wall_ew: sprite(176.0, 0.0, 30.0, 20.0),
};

const TURBINE_HALL: Block = Block {
    base: sprite(4.0, 38.0, 44.0, 22.0),
    height: 11.0,
    roof: sprite(80.0, 32.0, 44.0, 22.0),
    wall_ns: sprite(128.0, 24.0, 44.0, 11.0),
    wall_ew: sprite(176.0, 24.0, 22.0, 11.0),
};

const CONTROL_ROOM: Block = Block {
    base: sprite(4.0, 64.0, 20.0, 12.0),
    height: 8.0,
    roof: sprite(80.0, 56.0, 20.0, 12.0),
    wall_ns: sprite(128.0, 40.0, 20.0, 8.0),
    wall_ew: sprite(152.0, 40.0, 12.0, 8.0),
};

const BODY: Rect = sprite(208.0, 0.0, 10.0, 36.0);
const CAP: Rect = sprite(220.0, 0.0, 10.0, 10.0);

pub const MODEL: Model = Model {
    yard: sprite(0.0, 0.0, 80.0, 80.0),
    blocks: &[BOILER_HOUSE, TURBINE_HALL, CONTROL_ROOM],
    stacks: &[
        Stack { base: Vec2::new(62.0, 14.0), height: 40.0, body: BODY, cap: CAP, plume: true },
        Stack { base: Vec2::new(62.0, 32.0), height: 34.0, body: BODY, cap: CAP, plume: true },
    ],
};

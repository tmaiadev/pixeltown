const Tile = @import("./tile.zig");
const Scenario = @This();

tileset: [64][64]Tile,

pub fn init() Scenario {
    var tileset: [64][64]Tile = undefined;

    for (0..64) |y| {
        for (0..64) |x| {
            tileset[y][x] = Tile.init();
        }
    }

    return .{ .tileset = tileset };
}

pub fn update(_: *Scenario) void {}

pub fn draw(_: *const Scenario) void {}

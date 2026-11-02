const rl = @import("raylib");
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

pub fn draw(_: *const Scenario) void {
    for (0..64) |y| {
        for (0..64) |x| {
            const int_x: i32 = @intCast(x);
            const int_y: i32 = @intCast(y);
            const tile_size: i32 = 16;
            const color = if (@mod(int_x + int_y, 2) == 0) rl.Color.dark_green else rl.Color.green;

            rl.drawRectangle(int_x * tile_size, int_y * tile_size, tile_size, tile_size, color);
        }
    }
}

const std = @import("std");
const rl = @import("raylib");
const camera = @import("camera.zig");
const Tile = @import("./tile.zig");
const Scenario = @This();

const NUM_TILE_ROW: i32 = 256;

tileset: [NUM_TILE_ROW][NUM_TILE_ROW]Tile,

pub fn getMapSizeInPixels() i32 {
    const f_total: f32 = @as(f32, @floatFromInt(NUM_TILE_ROW)) * camera.scale;
    return @trunc(f_total);
}

pub fn init() Scenario {
    var tileset: [NUM_TILE_ROW][NUM_TILE_ROW]Tile = undefined;

    for (0..NUM_TILE_ROW) |y| {
        for (0..NUM_TILE_ROW) |x| {
            tileset[y][x] = Tile.init();
        }
    }

    return .{ .tileset = tileset };
}

pub fn update(_: *Scenario) void {}

pub fn draw(_: *const Scenario) void {
    const cam_rect = camera.rect();
    var tile_index: u32 = 0;

    for (0..NUM_TILE_ROW) |i_pos_y| {
        tile_index += 1;

        for (0..NUM_TILE_ROW) |i_pos_x| {
            tile_index += 1;

            const f_pos_x: f32 = @floatFromInt(i_pos_x);
            const f_pos_y: f32 = @floatFromInt(i_pos_y);

            const f_size: f32 = 16.0 * camera.scale;
            const f_x: f32 = f_pos_x * f_size;
            const f_y: f32 = f_pos_y * f_size;

            const draw_rect = rl.Rectangle.init(f_x, f_y, f_size, f_size);

            if (cam_rect.checkCollision(draw_rect)) {
                const i_size: i32 = @intFromFloat(f_size);
                const i_x: i32 = @intFromFloat(f_x);
                const i_y: i32 = @intFromFloat(f_y);

                const i_cam_offset_x: i32 = @as(i32, @intFromFloat(cam_rect.x)) * -1;
                const i_cam_offset_y: i32 = @as(i32, @intFromFloat(cam_rect.y)) * -1;

                const color = if (@mod(tile_index, 2) == 0) rl.Color.dark_green else rl.Color.green;
                rl.drawRectangle(i_x + i_cam_offset_x, i_y + i_cam_offset_y, i_size, i_size, color);
            }
        }
    }
}

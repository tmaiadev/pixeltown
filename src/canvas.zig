const std = @import("std");
const assert = std.debug.assert;
const rl = @import("raylib");
const window = @import("./window.zig");

const INNER_HEIGHT = 480;

var target: ?rl.RenderTexture = null;

pub fn init() !void {
    target = try rl.loadRenderTexture(width(), INNER_HEIGHT);
    rl.setTextureFilter(target.?.texture, .point);
}

pub fn deinit() void {
    assert(target != null);

    rl.unloadRenderTexture(target.?);
}

pub fn width() i32 {
    const f_width = @as(f32, @floatFromInt(window.width()));
    const f_height = @as(f32, @floatFromInt(window.height()));

    assert(f_width > 0);
    assert(f_height > 0);

    const scale: f32 = @as(f32, @floatFromInt(INNER_HEIGHT)) / f_height;
    return @intFromFloat(f_width * scale);
}

pub fn begin() void {
    assert(target != null);

    rl.beginTextureMode(target.?);
}

pub fn end() void {
    rl.endTextureMode();
}

pub fn draw() void {
    assert(target != null);

    const w: f32 = @floatFromInt(target.?.texture.width);
    const h: f32 = @floatFromInt(target.?.texture.height);

    const src = rl.Rectangle.init(0, 0, w, -h); // Render textures are stored upside down, hence the negative height
    const dst = rl.Rectangle.init(0, 0, @floatFromInt(window.width()), @floatFromInt(window.height()));

    rl.drawTexturePro(target.?.texture, src, dst, .{ .x = 0, .y = 0 }, 0, .white);
}

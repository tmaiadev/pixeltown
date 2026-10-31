const std = @import("std");
const rl = @import("raylib");

const Canvas = @This();

pub const internal_height = 480;

target: rl.RenderTexture,

pub fn init(window_width: i32, window_height: i32) !Canvas {
    const size = fitSize(window_width, window_height);
    const target = try rl.loadRenderTexture(size.width, size.height);
    rl.setTextureFilter(target.texture, .point);
    return .{ .target = target };
}

pub fn deinit(self: *Canvas) void {
    rl.unloadRenderTexture(self.target);
    self.* = undefined;
}

pub fn resize(self: *Canvas, window_width: i32, window_height: i32) !void {
    if (window_width <= 0 or window_height <= 0) return;

    const next = try init(window_width, window_height);
    self.deinit();
    self.* = next;
}

pub fn begin(self: Canvas) void {
    rl.beginTextureMode(self.target);
}

pub fn end(_: Canvas) void {
    rl.endTextureMode();
}

pub fn draw(self: Canvas, dst_width: i32, dst_height: i32) void {
    const w: f32 = @floatFromInt(self.target.texture.width);
    const h: f32 = @floatFromInt(self.target.texture.height);

    const src = rl.Rectangle.init(0, 0, w, -h); // Render textures are stored upside down, hence the negative height
    const dst = rl.Rectangle.init(0, 0, @floatFromInt(dst_width), @floatFromInt(dst_height));
    rl.drawTexturePro(self.target.texture, src, dst, .{ .x = 0, .y = 0 }, 0, .white);
}

const Size = struct { width: i32, height: i32 };

fn fitSize(window_width: i32, window_height: i32) Size {
    const scale = @as(f32, internal_height) / @as(f32, @floatFromInt(window_height));
    const w: i32 = @intFromFloat(@as(f32, @floatFromInt(window_width)) * scale);
    return .{ .width = @max(w, 1), .height = internal_height };
}

const rl = @import("raylib");
const canvas = @import("./canvas.zig");

pub var x: i32 = 0;
pub var y: i32 = 0;
pub var scale: f32 = 1;

pub fn reset() void {
    x = 0;
    y = 0;
    scale = 1;
}

pub fn width() i32 {
    return canvas.width();
}

pub fn height() i32 {
    return canvas.height();
}

pub fn rect() rl.Rectangle {
    return rl.Rectangle.init(@floatFromInt(x), @floatFromInt(y), @floatFromInt(width()), @floatFromInt(height()));
}

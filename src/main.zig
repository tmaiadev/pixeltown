const std = @import("std");
const rl = @import("raylib");

pub fn main() void {
    rl.initWindow(640, 480, "Pixeltown");
    defer rl.closeWindow();

    rl.setTargetFPS(60);

    while (!rl.windowShouldClose()) {
        rl.beginDrawing();
        rl.clearBackground(.black);
        rl.drawText("Hello, world!", 0, 0, 32, .white);
        rl.endDrawing();
    }
}

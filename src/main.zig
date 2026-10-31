const std = @import("std");
const rl = @import("raylib");

fn update() void {}

fn draw() void {
    rl.drawText("Hello, world!", 0, 0, 32, .white);
}

pub fn main() void {
    rl.initWindow(640, 480, "Pixeltown");
    defer rl.closeWindow();

    rl.setTargetFPS(60);

    while (!rl.windowShouldClose()) {
        update();

        rl.beginDrawing();
        rl.clearBackground(.black);
        draw();
        rl.endDrawing();
    }
}

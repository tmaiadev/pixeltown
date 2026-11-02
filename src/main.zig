const rl = @import("raylib");
const window = @import("window.zig");
const canvas = @import("canvas.zig");

fn update() void {}

fn draw() void {
    rl.drawText("Hello, world!", 0, 0, 32, .white);
}

pub fn main() !void {
    window.init();
    defer window.close();

    try canvas.init();
    defer canvas.deinit();

    while (!window.shouldClose()) {
        update();

        if (window.isResized()) {
            canvas.deinit();
            try canvas.init();
        }

        canvas.begin();
        rl.clearBackground(.black);
        draw();
        canvas.end();

        rl.beginDrawing();
        rl.clearBackground(.black);
        canvas.draw();
        rl.endDrawing();
    }
}

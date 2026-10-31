const rl = @import("raylib");
const window = @import("window.zig");
const Canvas = @import("canvas.zig");

fn update() void {}

fn draw() void {
    rl.drawText("Hello, world!", 0, 0, 32, .white);
}

pub fn main() !void {
    window.init();
    defer window.close();

    var canvas = try Canvas.init(window.width(), window.height());
    defer canvas.deinit();

    while (!window.shouldClose()) {
        update();

        if (window.isResized()) {
            try canvas.resize(window.width(), window.height());
        }

        canvas.begin();
        rl.clearBackground(.black);
        draw();
        canvas.end();

        rl.beginDrawing();
        rl.clearBackground(.black);
        canvas.draw(window.width(), window.height());
        rl.endDrawing();
    }
}

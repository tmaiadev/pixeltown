const rl = @import("raylib");
const window = @import("window.zig");
const canvas = @import("canvas.zig");
const GameplayStage = @import("stage/gameplay.zig");

pub fn main() !void {
    var stage = GameplayStage.init();

    window.init();
    defer window.close();

    try canvas.init();
    defer canvas.deinit();

    while (!window.shouldClose()) {
        stage.update();

        if (window.isResized()) {
            canvas.deinit();
            try canvas.init();
        }

        canvas.begin();
        rl.clearBackground(.black);
        stage.draw();
        canvas.end();

        rl.beginDrawing();
        rl.clearBackground(.black);
        canvas.draw();
        rl.endDrawing();
    }
}

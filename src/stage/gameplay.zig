const rl = @import("raylib");
const Scenario = @import("../scenario.zig");
const camera = @import("../camera.zig");
const Gameplay = @This();

scenario: Scenario,

pub fn init() Gameplay {
    return .{
        .scenario = Scenario.init(),
    };
}

pub fn update(self: *Gameplay) void {
    self.scenario.update();

    if (rl.isKeyPressed(rl.KeyboardKey.equal)) {
        if (camera.scale < 2) {
            camera.scale += 0.25;
        }
    }

    if (rl.isKeyPressed(rl.KeyboardKey.minus)) {
        if (camera.scale > 0.25) {
            camera.scale -= 0.25;
        }
    }
}

pub fn draw(self: *const Gameplay) void {
    self.scenario.draw();
}

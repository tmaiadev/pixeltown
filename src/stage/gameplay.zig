const std = @import("std");
const rl = @import("raylib");
const Scenario = @import("../scenario.zig");
const Animation = @import("../animation.zig");
const camera = @import("../camera.zig");
const Gameplay = @This();

const ZoomDirection = enum {
    In,
    Out,
};

scenario: Scenario,
zoomAni: ?Animation = null,
zoomAniDirection: ZoomDirection = .In,
zoomAniStartScale: ?f32 = null,

pub fn init() Gameplay {
    return .{
        .scenario = Scenario.init(),
    };
}

pub fn update(self: *Gameplay) void {
    self.scenario.update();

    if (rl.isKeyPressed(rl.KeyboardKey.equal)) {
        if (camera.scale < 2 and self.zoomAni == null) {
            self.zoomAni = Animation.init(0.25);
            self.zoomAniDirection = ZoomDirection.In;
            self.zoomAniStartScale = camera.scale;
        }
    }

    if (rl.isKeyPressed(rl.KeyboardKey.minus)) {
        if (camera.scale > 0.25 and self.zoomAni == null) {
            self.zoomAni = Animation.init(0.25);
            self.zoomAniDirection = ZoomDirection.Out;
            self.zoomAniStartScale = camera.scale;
        }
    }

    if (self.zoomAni != null) {
        self.zoomAni.?.update();

        const progress = self.zoomAni.?.getProgress();
        const dirValue: f32 = if (self.zoomAniDirection == ZoomDirection.In) 1.0 else -1.0;
        camera.scale = self.zoomAniStartScale.? + (progress * 0.25 * dirValue);

        if (self.zoomAni.?.hasEnded) {
            self.zoomAni = null;
            self.zoomAniStartScale = null;
        }
    }
}

pub fn draw(self: *const Gameplay) void {
    self.scenario.draw();
    rl.drawFPS(0, 0);
}

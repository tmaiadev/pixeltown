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

    if (rl.isKeyPressed(.equal)) {
        if (camera.scale < 2 and self.zoomAni == null) {
            self.zoomAni = Animation.init(0.25);
            self.zoomAniDirection = ZoomDirection.In;
            self.zoomAniStartScale = camera.scale;
        }
    }

    if (rl.isKeyPressed(.minus)) {
        if (camera.scale > 0.25 and self.zoomAni == null) {
            self.zoomAni = Animation.init(0.25);
            self.zoomAniDirection = ZoomDirection.Out;
            self.zoomAniStartScale = camera.scale;
        }
    }

    if (rl.isKeyDown(.right) or rl.isKeyDown(.d)) {
        camera.x += 4;
        const limit_x: i32 = self.scenario.getMapSize() - camera.width();

        if (camera.x > limit_x) {
            camera.x = limit_x;
        }
    }

    if (rl.isKeyDown(.left) or rl.isKeyDown(.a)) {
        camera.x -= 4;

        if (camera.x < 0) {
            camera.x = 0;
        }
    }

    if (rl.isKeyDown(.down) or rl.isKeyDown(.s)) {
        camera.y += 4;
        const limit_y: i32 = self.scenario.getMapSize() - camera.height();

        if (camera.y > limit_y) {
            camera.y = limit_y;
        }
    }

    if (rl.isKeyDown(.up) or rl.isKeyDown(.w)) {
        camera.y -= 4;

        if (camera.y < 0) {
            camera.y = 0;
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

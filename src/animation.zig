const rl = @import("raylib");

const Animation = @This();

duration: f32 = 0.00,
elapsed: f32 = 0.00,
loop: bool = false,
hasEnded: bool = false,
isPaused: bool = false,

pub fn init(duration: f32) Animation {
    return .{ .elapsed = 0.00, .duration = duration };
}

pub fn update(self: *Animation) void {
    if (self.hasEnded or self.isPaused) {
        return;
    }

    self.elapsed += rl.getFrameTime();

    if (self.loop and self.elapsed > self.duration) {
        self.elapsed = self.elapsed - self.duration;
    } else if (!self.loop and self.elapsed > self.duration) {
        self.elapsed = self.duration;
        self.isPaused = true;
        self.hasEnded = true;
    }
}

pub fn getProgress(self: *Animation) f32 {
    return self.elapsed / self.duration;
}

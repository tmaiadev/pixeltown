const rl = @import("raylib");

const Animation = @This();

duration: f32 = 0.00,
elapsed: f32 = 0.00,
loop: bool = false,
hasEnded: bool,
isPaused: bool,

pub fn init(self: *Animation, duration: f32) *Animation {
    self.elapsed = 0.00;
    self.duration = duration;
    return self;
}

pub fn update(self: *Animation) *Animation {
    if (self.hasEnded || self.isPaused) {
        return self;
    }

    self.elapsed += rl.getFrameTime();

    if (self.loop and self.elapsed > self.duration) {
        self.elapsed = self.elapsed - self.duration;
    } else if (!self.loop and self.elapsed > self.duration) {
        self.elapsed = self.duration;
        self.isPaused = true;
        self.hasEnded = true;
    }

    return self;
}

pub fn getProgress(self: *Animation) f32 {
    return self.elapsed / self.duration;
}

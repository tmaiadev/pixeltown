const Scenario = @import("../scenario.zig");
const Gameplay = @This();

scenario: Scenario,

pub fn init() Gameplay {
    return .{
        .scenario = Scenario.init(),
    };
}

pub fn update(self: *Gameplay) void {
    self.scenario.update();
}

pub fn draw(self: *const Gameplay) void {
    self.scenario.draw();
}

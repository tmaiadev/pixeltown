const rl = @import("raylib");

pub fn init() void {
    rl.setConfigFlags(.{
        .window_resizable = true,
    });

    rl.initWindow(640, 480, "Pixeltown");
    rl.setTargetFPS(60);

    const monitor = rl.getCurrentMonitor();
    rl.setWindowSize(rl.getMonitorWidth(monitor), rl.getMonitorHeight(monitor));
    rl.setWindowPosition(0, 0);
    rl.toggleBorderlessWindowed();
}

pub fn close() void {
    rl.closeWindow();
}

pub fn shouldClose() bool {
    return rl.windowShouldClose();
}

pub fn isResized() bool {
    return rl.isWindowResized();
}

pub fn width() i32 {
    return rl.getScreenWidth();
}

pub fn height() i32 {
    return rl.getScreenHeight();
}

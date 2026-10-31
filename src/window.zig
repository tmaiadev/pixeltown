const rl = @import("raylib");

pub fn init() void {
    rl.setConfigFlags(.{
        .fullscreen_mode = true,
        .window_resizable = true,
        .borderless_windowed_mode = true,
    });
    rl.initWindow(640, 480, "Pixeltown");
    rl.setTargetFPS(60);
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

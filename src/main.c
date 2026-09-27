#include "raylib.h"

const char *TITLE = "Pixeltown";

void update(void) {
}

void draw(void) {
    ClearBackground(BLACK);
    DrawText("Hello, world!", 0, 0, 16, WHITE);
}

int main(void) {
    InitWindow(256, 256, TITLE);
    SetWindowState(FLAG_WINDOW_RESIZABLE);
    SetTargetFPS(60);

    while (!WindowShouldClose()) {
        update();

        BeginDrawing();
        draw();
        EndDrawing();
    }

    CloseWindow();
    return 0;
}

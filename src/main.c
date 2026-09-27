#include "raylib.h"
#include <stdio.h>

const char *TITLE = "Pixeltown";

typedef struct {
    int w;
    int h;
} screen_size_t;

static screen_size_t get_screen_size() {
    int win_w = GetScreenWidth();
    int win_h = GetScreenHeight();
    float scale = (float)win_h / 256;
    int w = win_w / scale;
    int h = win_h / scale;

    return (screen_size_t){w,h};
}

static RenderTexture2D load_render_texture() {
    screen_size_t screen_size = get_screen_size();
    RenderTexture2D texture = LoadRenderTexture(screen_size.w, screen_size.h);

    return texture;
}

void update(void) {
}

void draw(void) {
    ClearBackground(BLACK);
    char text[100];
    screen_size_t screen_size = get_screen_size();
    snprintf(text, sizeof(text), "Hello, world! %dx%d", screen_size.w, screen_size.h);
    DrawText(text, 0, 0, 16, WHITE);
}

int main(void) {
    InitWindow(256, 256, TITLE);
    SetWindowState(FLAG_WINDOW_RESIZABLE);
    SetTargetFPS(60);

    RenderTexture2D canvas = load_render_texture();
    SetTextureFilter(canvas.texture, TEXTURE_FILTER_POINT);

    while (!WindowShouldClose()) {
        update();

        if (IsWindowResized()) canvas = load_render_texture();

        BeginTextureMode(canvas);
        draw();
        EndTextureMode();

        BeginDrawing();
        ClearBackground(BLACK);
        Rectangle src_rect = {0.0f, 0.0f, (float)canvas.texture.width, (float)canvas.texture.height * -1};
        Rectangle dst_rect = {0.0f, 0.0f, (float)GetScreenWidth(), (float)GetScreenHeight()};
        DrawTexturePro(canvas.texture, src_rect, dst_rect, (Vector2){0,0}, 0.1f, WHITE);
        EndDrawing();
    }

    CloseWindow();
    return 0;
}

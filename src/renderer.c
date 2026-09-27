#include "raylib.h"
#include "renderer.h"

const char *TITLE = "Pixeltown";

renderer_size_t renderer_get_size() {
    int win_w = GetScreenWidth();
    int win_h = GetScreenHeight();
    float scale = (float)win_h / 256;
    int w = win_w / scale;
    int h = win_h / scale;

    return (renderer_size_t){w,h};
}

static RenderTexture2D load_render_texture() {
    renderer_size_t screen_size = renderer_get_size();
    RenderTexture2D texture = LoadRenderTexture(screen_size.w, screen_size.h);

    return texture;
}

void renderer_init(void (update)(void), void (draw)(void)) {
    InitWindow(256, 256, TITLE);
    SetWindowState(FLAG_WINDOW_RESIZABLE);
    SetTargetFPS(60);

    RenderTexture2D canvas = load_render_texture();
    SetTextureFilter(canvas.texture, TEXTURE_FILTER_POINT);

    while (!WindowShouldClose()) {
        update();

        if (IsWindowResized()) canvas = load_render_texture();

        // We paint everything in `canvas` texture (fixed height 256px)
        BeginTextureMode(canvas);
        draw();
        EndTextureMode();

        // Resize the `canvas` texture to fill the Window, draw it
        BeginDrawing();
        ClearBackground(BLACK);
        Rectangle src_rect = {0.0f, 0.0f, (float)canvas.texture.width, (float)canvas.texture.height * -1};
        Rectangle dst_rect = {0.0f, 0.0f, (float)GetScreenWidth(), (float)GetScreenHeight()};
        DrawTexturePro(canvas.texture, src_rect, dst_rect, (Vector2){0,0}, 0.1f, WHITE);
        EndDrawing();
    }

    CloseWindow();
}

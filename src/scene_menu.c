#include "raylib.h"
#include "scene.h"
#include "screen.h"
#include <math.h>

static const unsigned char _bg_png[] = {
    #include "menu_bg.png.h"
};

Texture2D _bg;

static void init(void) {
    Image img = LoadImageFromMemory(".png", _bg_png, sizeof(_bg_png));
    _bg = LoadTextureFromImage(img);
    UnloadImage(img);
}

static void update(void) {

}

static void draw(void) {
    int n_bg_repeat = ceilf((float)screen_get_width() / screen_get_height());

    for (int i = 0; i < n_bg_repeat; i++) {
        int height = screen_get_height();

        Rectangle bg_rect = (Rectangle){
            .x = 0,
            .y = 0,
            .width = _bg.width,
            .height = _bg.height
        };

        Rectangle dist_rect = (Rectangle){
            .x = height * i,
            .y = 0,
            .width = height,
            .height = height,
        };

        DrawTexturePro(_bg, bg_rect, dist_rect, (Vector2){0,0}, 0.0f, WHITE);
    }
}

static void unload(void) {
}

scene_t scene_menu = {
    .init = init,
    .update = update,
    .draw = draw,
    .unload = unload
};

#include "ani.h"
#include "raylib.h"
#include "scene.h"
#include "screen.h"
#include <math.h>

static const unsigned char _bg_png[] = {
    #include "menu_bg.png.h"
};

static Texture2D _bg;
static ani_t _fade_in_ani;

static void init(void) {
    // Loads background image texture in `_bg`
    Image img = LoadImageFromMemory(".png", _bg_png, sizeof(_bg_png));
    _bg = LoadTextureFromImage(img);
    UnloadImage(img);

    // Initialises fade in animate
    _fade_in_ani = ani_create(1000);
}

static void update(void) {
    if (!_fade_in_ani.has_ended) {
        ani_update(&_fade_in_ani);
    }
}

static void draw(void) {
    Rectangle screen_size = screen_get_rect();

    // Draws background
    int n_bg_repeat = ceilf((float)screen_size.width / screen_size.height);

    for (int i = 0; i < n_bg_repeat; i++) {
        int height = screen_size.height;

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

    // Draws fade in animation
    DrawRectangle(0, 0, screen_size.width, screen_size.height, ColorAlpha(BLACK, 1.0f - _fade_in_ani.progress));
}

static void unload(void) {
}

scene_t scene_menu = {
    .init = init,
    .update = update,
    .draw = draw,
    .unload = unload
};

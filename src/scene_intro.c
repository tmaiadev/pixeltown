#include "ani.h"
#include "raylib.h"
#include "scene.h"
#include "scene_menu.h"
#include "text.h"
#include "screen.h"
#include <string.h>
#include <stdio.h>

static ani_t _anims[4];
static int _curr_ani_i = 0;

static void init(void) {
    _anims[0] = ani_create(1000); // BLANK
    _anims[1] = ani_create(4000); // PRESENTS
    _anims[2] = ani_create(4000); // TITLE
    _anims[3] = ani_create(1000); // BLANK
}

static void update(void) {
    ani_update(&_anims[_curr_ani_i]);

    if (_anims[_curr_ani_i].has_ended) {
        _curr_ani_i += 1;

        if (_curr_ani_i >= sizeof(_anims) / sizeof(_anims[0])) {
            scene_init(scene_menu);
        }
    }
}

static void draw(void) {
    Rectangle screen = screen_get_rect();

    if (_curr_ani_i == 1) {
        text_draw_center("@tmaiadev presents", screen_get_rect());
    }

    if (_curr_ani_i == 2) {
        text_draw_center("PixelTown", screen_get_rect());
    }

    float alpha = 1.0f;
    ani_t *curr_ani = &_anims[_curr_ani_i];

    if (curr_ani->elapsed_ms < 1000) {
        alpha = 1.0f - ((float)curr_ani->elapsed_ms / 1000);
    } else if (curr_ani->elapsed_ms < 3000) {
        alpha = 0.0f;
    } else if (curr_ani->elapsed_ms < 4000) {
        alpha = ((float)curr_ani->elapsed_ms - 3000) / 1000;
    } else {
        alpha = 1.0f;
    }

    DrawRectangle(0, 0, screen.width, screen.height, ColorAlpha(BLACK, alpha));
}

static void unload(void) {
}

scene_t scene_intro = {
    .init = init,
    .update = update,
    .draw = draw,
    .unload = unload
};

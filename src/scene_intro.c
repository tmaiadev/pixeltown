#include "ani.h"
#include "raylib.h"
#include "text.h"
#include "screen.h"
#include <string.h>
#include <stdio.h>

enum animations {
    WAIT,
    TEXT_A,
    TEXT_B,
    LOGO
};

char ani_id;
ani_t ani;
Font font;

void scene_intro_init(void) {
    ani_id = WAIT;
    ani = ani_create(1000);
}

void scene_intro_update(void) {
    ani_update(&ani);

    if (ani.has_ended && ani_id < LOGO) {
        ani_id += 1;
        ani = ani_create(5000);
    }
}

void scene_intro_draw(void) {
    Rectangle screen = screen_get_rect();

    if (ani_id == TEXT_A) {
        text_draw_center("@tmaiadev presents", screen_get_rect());
    }

    if (ani_id == TEXT_B) {
        text_draw_center("PixelTown", screen_get_rect());
    }

    float alpha = 1.0f;

    if (ani.elapsed_ms < 1000) {
        alpha = 1.0f - ((float)ani.elapsed_ms / 1000);
    } else if (ani.elapsed_ms < 3000) {
        alpha = 0.0f;
    } else if (ani.elapsed_ms < 4000) {
        alpha = ((float)ani.elapsed_ms - 3000) / 1000;
    } else {
        alpha = 1.0f;
    }

    DrawRectangle(0, 0, screen.width, screen.height, ColorAlpha(BLACK, alpha));
}

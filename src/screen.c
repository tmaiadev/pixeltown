#include "screen.h"

int screen_get_width(void) {
    int win_w = GetScreenWidth();
    int win_h = GetScreenHeight();

    float scale = (float)win_h / 256;
    int width = win_w / scale;

    return width;
};

int screen_get_height(void) {
    return 256;
};

Rectangle screen_get_rect(void) {
    return (Rectangle){
        .x = 0,
        .y = 0,
        .width = screen_get_width(),
        .height = screen_get_height(),
    };
};

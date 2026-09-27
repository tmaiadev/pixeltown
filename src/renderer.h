#pragma once

const char *TITLE;

typedef struct {
    int w;
    int h;
} renderer_size_t;

renderer_size_t renderer_get_size();
void renderer_init(void (update)(void), void (draw)(void));

#pragma once

#include "raylib.h"

void text_set_defaults(void);
void text_set_size(int size);
void text_set_color(Color color);
void text_draw(const char *text, int x, int y);
void text_draw_center(const char *text, Rectangle rect);

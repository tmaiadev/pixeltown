#include "raylib.h"
#include <stdbool.h>
#include <stddef.h>

static const unsigned char _font_ttf[] = {
    #include "jersey10.ttf.h"
};

static int _size;
static int _spacing;
static Color _color;
static Font _font;
static bool _defaults_called = false;


void text_set_defaults(void) {
    _size = 14;
    _spacing = 1;
    _color = WHITE;
    _font = LoadFontFromMemory(".ttf", _font_ttf, sizeof(_font_ttf), _size, NULL, 0);
    _defaults_called = true;
}

void text_set_size(int size) {
    _size = size;
}

void text_set_color(Color color) {
    _color = color;
}

void text_draw(const char *text, int x, int y) {
    if (_defaults_called == false) {
        text_set_defaults();
    }

    DrawTextEx(_font, text, (Vector2){x,y}, _size, _spacing, _color);
}

void text_draw_center(const char *text, Rectangle rect) {
    Vector2 size = MeasureTextEx(_font, text, _size, _spacing);
    text_draw(text, rect.width / 2 - size.x / 2, rect.height / 2 - size.y / 2);
}

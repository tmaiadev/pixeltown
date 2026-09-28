#import "raylib.h"

static int _size = 10;
static int _spacing = 1;
static Color _color = WHITE;

void text_set_defaults(void) {
    _size = 10;
    _spacing = 1;
    _color = WHITE;
}

void text_set_size(int size) {
    _size = size;
}

void text_set_color(Color color) {
    _color = color;
}

void text_draw(const char *text, int x, int y) {
    DrawTextEx(GetFontDefault(), text, (Vector2){x,y}, _size, _spacing, _color);
}

void text_draw_center(const char *text, Rectangle rect) {
    Vector2 size = MeasureTextEx(GetFontDefault(), text, _size, _spacing);
    text_draw(text, rect.width / 2 - size.x / 2, rect.height / 2 - size.y / 2);
}

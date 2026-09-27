#include "raylib.h"
#include "renderer.h"
#include <stdio.h>

void update(void) {
}

void draw(void) {
    ClearBackground(BLACK);
    char text[100];
    renderer_size_t screen_size = renderer_get_size();
    snprintf(text, sizeof(text), "Hello, world! %dx%d", screen_size.w, screen_size.h);
    DrawText(text, 0, 0, 16, WHITE);
}

int main(void) {
   renderer_init(update, draw);
   return 0;
}

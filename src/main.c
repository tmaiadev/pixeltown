#include "renderer.h"
#include "scene_intro.h"
#include <stdio.h>

void update(void) {
    scene_intro_update();
}

void draw(void) {
    scene_intro_draw();
}

int main(void) {
    scene_intro_init();
    renderer_init(update, draw);
    return 0;
}

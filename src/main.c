#include "renderer.h"
#include "scene.h"
#include "scene_intro.h"
#include <stdio.h>

void update(void) {
    scene_update();
}

void draw(void) {
    scene_draw();
}

int main(void) {
    scene_init(scene_intro);
    renderer_init(update, draw);
    return 0;
}

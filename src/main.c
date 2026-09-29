#include "renderer.h"
#include "scene.h"
#include "scene_intro.h"
#include <stdio.h>

int main(void) {
    scene_init(scene_intro);
    renderer_init(scene_update, scene_draw);
    return 0;
}

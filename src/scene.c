#include <stdbool.h>
#include <stdio.h>
#include "scene.h"

static scene_t _scene;
static bool _initialised;

void scene_init(scene_t scene) {
    _initialised = true;
    _scene = scene;

    _scene.init();
}

void scene_update(void) {
    if (_initialised == false) {
        printf("ERROR: `scene_update` ran before scene was initialised");
        return;
    }

    _scene.update();
}

void scene_draw(void) {
    _scene.draw();
}

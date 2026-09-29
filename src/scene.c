#include <stdbool.h>
#include <stdio.h>
#include "scene.h"

static scene_t _scene;
static bool _initialised;

void scene_init(scene_t scene) {
    if (_initialised == true) {
        // If this has been already initialised,
        // it means we have to unload the previous
        // scene before we initialise a new one.
        _scene.unload();
    }

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

#pragma once

typedef struct {
    void (*init)(void);
    void (*update)(void);
    void (*draw)(void);
} scene_t;

void scene_init(scene_t scene);
void scene_update(void);
void scene_draw(void);

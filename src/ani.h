#pragma once

#include <stdbool.h>

typedef struct {
    int duration_ms;
    bool will_loop;
    bool is_playing;
    int elapsed_ms;
    float progress;
    bool has_ended;
} ani_t;

ani_t ani_create(int duration_ms);
void ani_update(ani_t *ani);

#include <stdbool.h>
#include <stdio.h>
#include "raylib.h"
#include "ani.h"

ani_t ani_create(int duration_ms) {
  return (ani_t){
      .duration_ms = duration_ms,
      .will_loop = false,
      .is_playing = true,
      .elapsed_ms = 0,
      .progress = 0,
      .has_ended = false,
  };
};

void ani_update(ani_t *ani) {
    if (ani->is_playing == false) return;

    ani->elapsed_ms += GetFrameTime() * 1000;

    if (ani->elapsed_ms > ani->duration_ms) {
        if (ani->will_loop) {
            ani->elapsed_ms = ani->elapsed_ms-ani->duration_ms;
            ani->progress = (float) ani->elapsed_ms / ani->duration_ms;
        } else {
            ani->is_playing = false;
            ani->elapsed_ms = ani->duration_ms;
            ani->progress = 1.0f;
            ani->has_ended = true;
        }
    } else {
        ani->progress = (float) ani->elapsed_ms / ani->duration_ms;
    }
}

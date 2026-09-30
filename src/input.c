#include "raylib.h"
#include "input.h"
#include <stdbool.h>

bool input_is_pressed(input_t input) {
    bool result = false;

    switch (input) {
        case INPUT_DISMISS:
            result = IsKeyPressed(KEY_ESCAPE);
            break;
        case INPUT_ACCEPT:
            result = IsKeyPressed(KEY_ENTER);
            break;
        default:
        break;
    }

  return result;
};

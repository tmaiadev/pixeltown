#pragma once

typedef enum {
    INPUT_NULL=0,
    INPUT_DISMISS,
    INPUT_ACCEPT,
} input_t;

bool input_is_pressed(input_t input);

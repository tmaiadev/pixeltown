#include "scene.h"
#include "screen.h"
#include "text.h"

static void init(void) {

}

static void update(void) {

}

static void draw(void) {
    text_draw_center("MENU", screen_get_rect());
}

scene_t scene_menu = {
    .init = init,
    .update = update,
    .draw = draw,
};

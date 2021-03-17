// I'm gonna go nuts from all this noise...
#![allow(dead_code)]

#[macro_use]
extern crate nom;

#[macro_use]
extern crate enum_display_derive;

use macroquad::*;
mod bloom;

#[macroquad::main("BasicShapes")]
async fn main() {
    loop {
        clear_background(BLACK);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);

        next_frame().await
    }
}

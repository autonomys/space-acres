use gtk::prelude::*;
use gtk::{glib, DrawingArea};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

/// props for drawing a circular progress bar
pub fn create_circular_progress_bar(
    diameter: f64,
    margin_top: i32,
    margin_bottom: i32,
    margin_start: i32,
    margin_end: i32,
    tooltip_text: &str,
) -> DrawingArea {
    // Create a shared state for the progress
    let progress = Rc::new(RefCell::new(1.0)); // Start fully "unwiped"

    let drawing_area = Rc::new(RefCell::new(
        DrawingArea::builder()
            .content_width((diameter + 1.0) as i32)
            .content_height((diameter + 1.0) as i32)
            .margin_top(margin_top)
            .margin_bottom(margin_bottom)
            .margin_start(margin_start)
            .margin_end(margin_end)
            .tooltip_text(tooltip_text)
            .build(),
    ));

    drawing_area.borrow().set_draw_func({
        let progress = progress.clone();
        move |_, cr, width, height| {
            let percentage = *progress.borrow();

            // Center coordinates
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;

            // Draw the full circle with respective color in dark/light themes
            if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                // Grey for dark theme
                cr.set_source_rgb(0.2078431373, 0.2078431373, 0.2078431373);
            } else {
                // White for light theme
                cr.set_source_rgb(1.0, 1.0, 1.0); // White for full circle
            }
            cr.arc(center_x, center_y, diameter / 2.0, 0.0, 2.0 * PI);
            // let _ = cr.fill();       // NOTE: Fill w/o border color
            let _ = cr.fill_preserve(); // Preserve the path for stroking
            cr.set_source_rgb(0.0, 0.0, 0.0); // Black for the border
            cr.set_line_width(0.5); // Set the border width
            let _ = cr.stroke(); // Draw the border

            // Draw the sweeping with respective color in dark/light themes
            if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                // White for dark theme
                cr.set_source_rgb(1.0, 1.0, 1.0);
            } else {
                // Grey for light theme
                cr.set_source_rgb(0.2078431373, 0.2078431373, 0.2078431373);
            }
            cr.arc(
                center_x,
                center_y,
                diameter / 2.0,
                -PI / 2.0,
                -PI / 2.0 + 2.0 * PI * percentage,
            );
            cr.line_to(center_x, center_y);
            let _ = cr.fill();
        }
    });

    // Update the progress i.e. sweep arc every `interval` second
    glib::timeout_add_seconds_local(1, {
        let progress = progress.clone();
        let drawing_area = drawing_area.clone();
        move || {
            let mut percentage = progress.borrow_mut();
            if (*percentage - 0.1) <= 0.0 {
                *percentage = 1.0; // Reset to full when it reaches 0
            } else {
                *percentage -= 0.1; // Decrease by 10%
            }
            drawing_area.borrow().queue_draw();
            glib::ControlFlow::Continue
        }
    });

    let progress_bar = drawing_area.borrow().clone();
    progress_bar
}

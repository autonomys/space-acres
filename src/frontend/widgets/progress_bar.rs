use gtk::prelude::*;
use gtk::DrawingArea;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

/// Create a circular progress bar
pub fn create_circular_progress_bar(
    diameter: f64,
    margin_top: i32,
    margin_bottom: i32,
    margin_start: i32,
    margin_end: i32,
    set_visible: bool,
    tooltip_text: &str,
    progress: Rc<RefCell<f64>>,
) -> DrawingArea {
    let drawing_area = DrawingArea::builder()
        .content_width((diameter + 1.0) as i32)
        .content_height((diameter + 1.0) as i32)
        .margin_top(margin_top)
        .margin_bottom(margin_bottom)
        .margin_start(margin_start)
        .margin_end(margin_end)
        .tooltip_text(tooltip_text)
        .visible(set_visible)
        .build();

    drawing_area.set_draw_func({
        let percentage = *progress.borrow();
        move |_, cr, width, height| {
            // Center coordinates
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;

            // Draw the background circle with respective color in dark/light themes
            if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                // Grey for dark theme
                cr.set_source_rgb(0.2078431373, 0.2078431373, 0.2078431373);
            } else {
                // White for light theme
                cr.set_source_rgb(1.0, 1.0, 1.0);
            }
            cr.arc(center_x, center_y, diameter / 2.0, 0.0, 2.0 * PI);
            // let _ = cr.fill();       // NOTE: Fill w/o border color
            let _ = cr.fill_preserve(); // Preserve the path for stroking
            cr.set_source_rgb(0.0, 0.0, 0.0); // Black border color for both the themes.
            cr.set_line_width(0.5); // Set the border width
            let _ = cr.stroke(); // Draw the circle border

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

    drawing_area
}

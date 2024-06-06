use gtk::prelude::*;
use relm4::prelude::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(super) struct CircularProgressBar {
    pub(super) progress: Rc<RefCell<f64>>,
    pub(super) tooltip_text: String,
    pub(super) diameter: f64,
    pub(super) margin_top: i32,
    pub(super) margin_bottom: i32,
    pub(super) margin_start: i32,
    pub(super) margin_end: i32,
    pub(super) is_visible: bool,
}

#[derive(Debug)]
pub enum CircularProgressBarInput {
    SetProgress(f64),
    SetTooltip(String),
}

#[relm4::component(pub)]
impl Component for CircularProgressBar {
    // Diameter of the progress bar
    type Init = CircularProgressBar;
    type Input = CircularProgressBarInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::DrawingArea {
            set_content_width: (model.diameter + 1.0) as i32,
            set_content_height: (model.diameter + 1.0) as i32,
            set_margin_top: model.margin_top,
            set_margin_bottom: model.margin_bottom,
            set_margin_start: model.margin_start,
            set_margin_end: model.margin_end,
            set_visible: true,
            set_draw_func: {
                let progress = model.progress.clone();
                let diameter = model.diameter;
                move |drawing_area, cr, width, height| {
                    let percentage = *progress.borrow();

                    // Center coordinates
                    let center_x = width as f64 / 2.0;
                    let center_y = height as f64 / 2.0;

                    let color = drawing_area.style_context().color();
                    let is_dark_theme = color.red() < 0.5 && color.green() < 0.5 && color.blue() < 0.5;

                    // Draw the background circle with respective color in dark/light themes
                    if !is_dark_theme {
                        // Grey for light theme
                        cr.set_source_rgb(0.2078431373, 0.2078431373, 0.2078431373);
                    } else {
                        // White smoke for dark theme
                        cr.set_source_rgb(0.965, 0.961, 0.957);
                    }
                    cr.arc(center_x, center_y, diameter / 2.0, 0.0, 2.0 * PI);
                    // let _ = cr.fill();       // NOTE: Fill w/o border color
                    let _ = cr.fill_preserve(); // Preserve the path for stroking
                    cr.set_source_rgb(0.0, 0.0, 0.0); // Black border color for both the themes.
                    cr.set_line_width(0.5); // Set the border width
                    let _ = cr.stroke(); // Draw the circle border

                    // Draw the sweeping with respective color in dark/light themes
                    if !is_dark_theme {
                        // White smoke for light theme
                        cr.set_source_rgb(0.965, 0.961, 0.957);
                    } else {
                        // Grey for dark theme
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
            },
            #[watch]
            set_tooltip_text: Some(&model.tooltip_text),
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            progress: init.progress,
            tooltip_text: init.tooltip_text,
            diameter: init.diameter,
            margin_top: init.margin_top,
            margin_bottom: init.margin_bottom,
            margin_start: init.margin_start,
            margin_end: init.margin_end,
            is_visible: init.is_visible,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match input {
            CircularProgressBarInput::SetProgress(p) => {
                *self.progress.borrow_mut() = p;
                root.queue_draw();
            }
            CircularProgressBarInput::SetTooltip(text) => {
                self.tooltip_text = text;
                root.set_tooltip_text(Some(&self.tooltip_text));
            }
        }
    }
}

/// Calculate the decrease value and interval for the progress updates
pub(crate) fn calculate_progress_params(eta_in_secs: u64) -> (f64, u64) {
    let interval = 1; // Default interval of 1 second (1000 milliseconds)
    let steps = eta_in_secs / interval;
    let decrease_value = 1.0 / steps as f64;
    (decrease_value, interval)
}

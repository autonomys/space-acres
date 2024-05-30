use gtk::prelude::*;
use relm4::prelude::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

pub(crate) const DEFAULT_TOOLTIP_ETA_PROGRESS_BAR: &str = "ETA for next reward payment";

#[derive(Debug, Clone)]
pub struct CircularProgressBar {
    progress: Rc<RefCell<f64>>,
    tooltip_text: String,
    diameter: f64,
}

#[derive(Debug)]
pub enum ProgressBarInput {
    SetProgress(f64),
    SetTooltip(String),
}

#[relm4::component(pub)]
impl Component for CircularProgressBar {
    type Init = f64; // Diameter of the progress bar
    type Input = ProgressBarInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::DrawingArea {
            set_content_width: (model.diameter + 1.0) as i32,
            set_content_height: (model.diameter + 1.0) as i32,
            set_margin_top: 10,
            set_margin_bottom: 10,
            set_margin_start: 10,
            set_margin_end: 10,
            set_visible: true,
            set_draw_func: {
                let progress_clone = model.progress.clone();
                let diameter = model.diameter;
                move |_, cr, width, height| {
                    let percentage = *progress_clone.borrow();

                    // Center coordinates
                    let center_x = width as f64 / 2.0;
                    let center_y = height as f64 / 2.0;

                    // Draw the background circle with respective color in dark/light themes
                    if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                        // Grey for dark theme
                        cr.set_source_rgb(0.2078431373, 0.2078431373, 0.2078431373);
                    } else {
                        // White smoke for light theme
                        cr.set_source_rgb(0.965, 0.961, 0.957);
                    }
                    cr.arc(center_x, center_y, diameter / 2.0, 0.0, 2.0 * PI);
                    // let _ = cr.fill();       // NOTE: Fill w/o border color
                    let _ = cr.fill_preserve(); // Preserve the path for stroking
                    cr.set_source_rgb(0.0, 0.0, 0.0); // Black border color for both the themes.
                    cr.set_line_width(0.5); // Set the border width
                    let _ = cr.stroke(); // Draw the circle border

                    // Draw the sweeping with respective color in dark/light themes
                    if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                        // White smoke for dark theme
                        cr.set_source_rgb(0.965, 0.961, 0.957);
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
            },
            set_tooltip_text: Some(&model.tooltip_text),
        }
    }

    fn init(
        diameter: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let progress = Rc::new(RefCell::new(0.0));
        let tooltip_text = DEFAULT_TOOLTIP_ETA_PROGRESS_BAR.to_string();

        let model = Self {
            progress,
            tooltip_text,
            diameter,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match input {
            ProgressBarInput::SetProgress(p) => {
                *self.progress.borrow_mut() = p;
                root.queue_draw();
            }
            ProgressBarInput::SetTooltip(text) => {
                self.tooltip_text = text;
                root.set_tooltip_text(Some(&self.tooltip_text));
            }
        }
    }
}

/// Calculate the decrease value and interval for the progress updates
pub(crate) fn calculate_progress_params(eta: u64) -> (f64, u64) {
    let interval = 1000; // Default interval of 1 second (1000 milliseconds)
    let steps = eta / interval;
    let decrease_value = 1.0 / steps as f64;
    (decrease_value, interval)
}

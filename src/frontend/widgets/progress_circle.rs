use gtk::prelude::*;
use relm4::prelude::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(in super::super) struct ProgressCircleInit {
    pub(in super::super) tooltip: String,
    pub(in super::super) size: u16,
}

#[derive(Debug)]
pub(in super::super) enum ProgressCircleInput {
    Update {
        /// 0.0..=1.0
        progress: f64,
        tooltip: String,
    },
}

#[derive(Debug, Clone)]
pub(in super::super) struct ProgressCircle {
    /// 0.0..=1.0
    pub(in super::super) progress: Rc<RefCell<f64>>,
    pub(in super::super) tooltip: String,
    pub(in super::super) size: u16,
}

#[relm4::component(pub)]
impl Component for ProgressCircle {
    type Init = ProgressCircleInit;
    type Input = ProgressCircleInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::DrawingArea {
            set_content_width: i32::from(model.size),
            set_content_height: i32::from(model.size),
            set_draw_func: {
                let progress = model.progress.clone();
                let size = f64::from(model.size);

                move |drawing_area, cr, width, height| {
                    let progress = *progress.borrow();

                    let color = drawing_area.style_context().color();

                    // Center coordinates
                    let center_x = width as f64 / 2.0;
                    let center_y = height as f64 / 2.0;

                    // Clear everything
                    cr.set_operator(gtk::cairo::Operator::Clear);
                    let _ = cr.paint();

                    cr.set_operator(gtk::cairo::Operator::Over);
                    let circle_color = color;
                    cr.set_source_rgba(
                        f64::from(circle_color.red()),
                        f64::from(circle_color.green()),
                        f64::from(circle_color.blue()),
                        f64::from(circle_color.alpha()),
                    );

                    // Draw outer border
                    let border_width = (size * 0.05).max(1.0);
                    cr.arc(center_x, center_y, size / 2.0 - border_width / 2.0, 0.0, 2.0 * PI);
                    cr.set_line_width(border_width);
                    let _ = cr.stroke();

                    // Draw circular progress
                    // Radians start at the east, hence `0.5 * PI` difference
                    cr.arc(
                        center_x,
                        center_y,
                        size / 2.0,
                        -0.5 * PI + 2.0 * PI * progress,
                        1.5 * PI,
                    );
                    cr.line_to(center_x, center_y);
                    let _ = cr.fill();
                }
            },
            #[watch]
            set_tooltip_text: Some(&model.tooltip),
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            progress: Rc::new(RefCell::new(0.0)),
            tooltip: init.tooltip,
            size: init.size,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match input {
            ProgressCircleInput::Update { progress, tooltip } => {
                let old_progress = *self.progress.borrow();
                if progress != old_progress {
                    *self.progress.borrow_mut() = progress;
                    if (progress - old_progress).abs() > 0.01 {
                        root.queue_draw();
                    }
                }

                self.tooltip = tooltip;
            }
        }
    }
}

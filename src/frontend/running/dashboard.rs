use gtk::prelude::*;
use relm4::prelude::*;
use relm4::ComponentSender;

#[derive(Debug)]
pub struct Dashboard {
    pub status_label: gtk::Label,
    pub farm_size_label: gtk::Label,
    pub tokens_earned_label: gtk::Label,
    pub time_to_win_label: gtk::Label,
    pub tokens_earned_24h_label: gtk::Label,
}

#[relm4::component(pub)]
impl SimpleComponent for Dashboard {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_start: 20,
            set_spacing: 5,

            gtk::Label {
                set_text: "Farms overview:",
                set_margin_all: 5,
                set_halign: gtk::Align::Start,
                add_css_class: "dashboard-title",
            },
            #[name(status_label)]
            gtk::Label {
                set_markup: "<span color='green'>farming</span> &amp; <span color='orange'>replotting</span>",
                set_margin_all: 5,
                set_halign: gtk::Align::Start,
                add_css_class: "status-label",
            },

            //main overview grid
            gtk::Grid {
                set_row_spacing: 10,
                set_column_spacing: 20,

                attach[2, 0, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_start: 10,
                    add_css_class: "last-day-label",

                    gtk::Label {
                        set_text: "In last 24 hours",
                        set_halign: gtk::Align::Start,
                    },
                },

                attach[0, 1, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "data-box",

                    gtk::Label {
                        set_text: "2 Farms, Total Size:",
                        set_halign: gtk::Align::Start,
                    },
                    #[name(farm_size_label)]
                    gtk::Label {
                        set_text: "120 GB",
                        set_halign: gtk::Align::Start,
                        add_css_class: "data-label",
                    }
                },

                attach[1, 1, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "button-box",

                    gtk::Button {
                        set_label: "Change setup",
                        add_css_class: "dashboard-button",
                    },
                    gtk::Button {
                        set_label: "Add new farm",
                        add_css_class: "dashboard-button",
                    }
                },

                attach[2, 1, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "data-box",

                    gtk::Label {
                        set_text: "Missed challenges:",
                        set_halign: gtk::Align::Start,
                    },
                    #[name(tokens_earned_label)]
                    gtk::Label {
                        set_text: "2",
                        set_halign: gtk::Align::Start,
                        add_css_class: "data-label", 
                        add_css_class: "orange",
                    }
                },

                attach[3, 1, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "button-box",

                    gtk::Button {
                        set_label: "Check farm issues",
                        add_css_class: "dashboard-button",
                    },
                },

                attach[0, 2, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "data-box",

                    gtk::Label {
                        set_text: "Tokens available:",
                        set_halign: gtk::Align::Start,
                    },
                    #[name(time_to_win_label)]
                    gtk::Label {
                        set_text: "200 ATC",
                        set_halign: gtk::Align::Start,
                        add_css_class: "data-label",
                    }
                },

                attach[1, 2, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "button-box",

                    gtk::Button {
                        set_label: "Withdraw funds",
                        add_css_class: "dashboard-button",
                    },
                    gtk::Button {
                        set_label: "Stake/Nominate",
                        add_css_class: "dashboard-button",
                    }
                },

                attach[2, 2, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "data-box",

                    gtk::Label {
                        set_text: "Tokens Earned:",
                        set_halign: gtk::Align::Start,
                    },
                    #[name(tokens_earned_24h_label)]
                    gtk::Label {
                        set_text: "0 ATC",
                        set_halign: gtk::Align::Start,
                        add_css_class: "data-label",
                    }
                },

                attach[3, 2, 1, 1] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    add_css_class: "button-box",

                    gtk::Button {
                        set_label: "Check ETA",
                        add_css_class: "dashboard-button",
                    },
                },
            },

            //test code below: expander within the main box

            #[name(farm_stats)]
            gtk::Expander {
                set_label: Some("Farm 1: 40 GB, detailed stats"),
                set_expanded: false,
                add_css_class: "farm-stats",
            
                //add contents to the expander
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    //setup button
                    gtk::Button {
                        set_label: "Change setup",
                        set_halign: gtk::Align::End,
                        add_css_class: "dashboard-button",
                    },      
                        
                    //details grid    
                    gtk::Grid {
                        set_row_spacing: 5,
                        set_column_spacing: 20,
                    
                        attach[0, 0, 1, 1] = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            add_css_class: "stat-box",
        
                            gtk::Label {
                                set_text: "Path to the farm:",
                                set_halign: gtk::Align::Start,
                            },
                            #[name(path_label)]
                            gtk::Label {
                                set_text: "C:Users",
                                set_halign: gtk::Align::Start,
                                add_css_class: "stat-label",
                            }
                        },

                        attach[1, 0, 1, 1] = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            add_css_class: "stat-box",
        
                            gtk::Label {
                                set_text: "Total tokens earned:",
                                set_halign: gtk::Align::Start,
                            },
                            #[name(farm_tokens_earned_label)]
                            gtk::Label {
                                set_text: "23 ATC",
                                set_halign: gtk::Align::Start,
                                add_css_class: "stat-label",
                            }
                        },

                        attach[0, 1, 1, 1] = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            add_css_class: "stat-box",
        
                            gtk::Label {
                                set_text: "Farm status:",
                                set_halign: gtk::Align::Start,
                            },
                            #[name(farming_status_label)]
                            gtk::Label {
                                set_text: "Farming",
                                set_halign: gtk::Align::Start,
                                add_css_class: "stat-label",
                                add_css_class: "green",
                            }
                        },

                        attach[1, 1, 1, 1] = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            add_css_class: "stat-box",
        
                            gtk::Label {
                                set_text: "Number of missed challenges last week:",
                                set_halign: gtk::Align::Start,
                            },
                            #[name(missed_challenges_label)]
                            gtk::Label {
                                set_text: "2, affecting 0.1% of earning potential",
                                set_halign: gtk::Align::Start,
                                add_css_class: "stat-label",
                                add_css_class: "orange",
                            }
                        },
                    },

                    //detailed logs expander
                    gtk::Expander {
                        set_label: Some("Detailed statistics and logs"),
                        set_expanded: false,
                        add_css_class: "detailed-stats",

                        gtk::Grid {
                            set_row_spacing: 5,
                            set_column_spacing: 20,
                            
                            attach[0, 0, 1, 1] = &gtk::Button {
                                set_label: "Download logs",
                                set_halign: gtk::Align::End,
                                add_css_class: "dashboard-button",
                            },   

                            attach[1, 0, 1, 1] = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                add_css_class: "stat-box",
            
                                gtk::Label {
                                    set_markup: "<span color='green'>•</span> Proof time:",
                                    set_halign: gtk::Align::Start,
                                },
                                #[name(proof_time_label)]
                                gtk::Label {
                                    set_text: "0.09 out of 1.0 s",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "detail-stat-label",
                                },
                                gtk::Label {
                                    set_markup: "<span color='green'>•</span> Audit time:",
                                    set_halign: gtk::Align::Start,
                                },
                                #[name(audit_time_label)]
                                gtk::Label {
                                    set_text: "0.01 out of 1.0 s",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "detail-stat-label",
                                },
                                gtk::Label {
                                    set_markup: "<span color='green'>•</span> Up time:",
                                    set_halign: gtk::Align::Start,
                                },
                                #[name(up_time_label)]
                                gtk::Label {
                                    set_text: "0.01 out of 1.0 s",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "detail-stat-label",
                                },
                            },
                            
                            attach[2, 0, 1, 1] = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                add_css_class: "stat-box",
            
                                gtk::Label {
                                    set_markup: "<span color='orange'>•</span> Plots status:",
                                    set_halign: gtk::Align::Start,
                                },
                                #[name(outdated_plots_label)]
                                gtk::Label {
                                    set_text: "2.5% outdated",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "detail-stat-label",
                                },
                                #[name(currently_replotting_label)]
                                gtk::Label {
                                    set_text: "3% currently replotting",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "detail-stat-label",
                                },
                                #[name(farming_plots_label)]
                                gtk::Label {
                                    set_text: "92% farming",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "detail-stat-label",
                                },
                            },

                            attach[1, 1, 1, 1] = &gtk::Button {
                                set_label: "Read how to improve",
                                set_halign: gtk::Align::Start,
                                add_css_class: "dashboard-button",
                            },   

                            attach[2, 1, 1, 1] = &gtk::Button {
                                set_label: "Start replotting",
                                set_halign: gtk::Align::Start,
                                add_css_class: "dashboard-button",
                            },   
                        },
                    },

                },
            },
        },
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Dashboard>,
    ) -> ComponentParts<Self> {
        let model = Dashboard {
            status_label: gtk::Label::new(Some("Current status: sync in progress")),
            farm_size_label: gtk::Label::new(Some("120 GB")),
            tokens_earned_label: gtk::Label::new(Some("0 ATC")),
            time_to_win_label: gtk::Label::new(Some("12 h")),
            tokens_earned_24h_label: gtk::Label::new(Some("0 ATC")),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

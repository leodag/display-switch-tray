use gtk::glib;
use gtk::glib::SignalHandlerId;
use gtk::prelude::*;
use std::path::Path;
use std::process::Command;
use std::cell::RefCell;
use std::rc::Rc;
use std::env;
use appindicator3::prelude::*;
use appindicator3::{Indicator, IndicatorStatus, IndicatorCategory};

const APP_NAME: &str = "Display Switch Tray";

#[derive(Debug)]
enum ActiveStateEnum {
    Inactive,
    Active,
}

#[derive(Debug)]
enum FailedStateEnum {
    Failed,
    Normal,
}

#[derive(Debug)]
struct ServiceState {
    active: ActiveStateEnum,
    failed: FailedStateEnum,
}

impl ServiceState {
    fn new(active_state: ActiveStateEnum, failed_state: FailedStateEnum) -> Self {
        ServiceState { active: active_state, failed: failed_state }
    }

    fn sync_state(&mut self, active_state: ActiveStateEnum, failed_state: FailedStateEnum) {
        self.active = active_state;
        self.failed = failed_state;
    }

    fn change_state(&mut self, desired_state: ActiveStateEnum) -> FailedStateEnum {
        let success = match desired_state {
            ActiveStateEnum::Active => {
                let status = Command::new("sudo")
                    .args(["systemctl", "start", "display_switch.service"])
                    .status()
                    .expect("failed to start");

                status.code().unwrap_or(1) == 0
            },
            ActiveStateEnum::Inactive => {
                let status = Command::new("sudo")
                    .args(["systemctl", "stop", "display_switch.service"])
                    .status()
                    .expect("failed to stop");

                status.code().unwrap_or(1) == 0
            }
        };

        if success {
            self.active = desired_state;
            self.failed = FailedStateEnum::Normal;
            FailedStateEnum::Normal
        } else {
            self.failed = FailedStateEnum::Failed;
            FailedStateEnum::Failed
        }
    }
}

fn current_unit_state() -> (ActiveStateEnum, FailedStateEnum) {
    let active = Command::new("systemctl")
        .args(["is-active", "--quiet", "display_switch.service"])
        .status()
        .expect("failed to check active");

    let failed = Command::new("systemctl")
        .args(["is-failed", "--quiet", "display_switch.service"])
        .status()
        .expect("failed to check failed");

    let active_state = match active.code() {
        Some(0) => ActiveStateEnum::Active,
        _ => ActiveStateEnum::Inactive,
    };

    let failed_state = match failed.code() {
        Some(0) => FailedStateEnum::Failed,
        _ => FailedStateEnum::Normal,
    };

    (active_state, failed_state)
}

fn set_icon(indicator: &Indicator, service_state: &ServiceState) {
    match service_state.failed {
        FailedStateEnum::Failed => {
            indicator.set_status(IndicatorStatus::Attention);
        },
        FailedStateEnum::Normal => {
            match service_state.active {
                ActiveStateEnum::Active => {
                    indicator.set_icon_full("active-symbolic", "icon");
                    indicator.set_status(IndicatorStatus::Active);
                },
                ActiveStateEnum::Inactive => {
                    indicator.set_icon_full("inactive-symbolic", "icon");
                    indicator.set_status(IndicatorStatus::Active);
                }
            }
        }
    }
}

fn main() {
    gtk::init().unwrap();

    let icons_dir = env::var("CARGO_MANIFEST_DIR")
        .unwrap_or(String::from("/usr/share/display-switch-tray/icons/"));
    let icon_path = Path::new(icons_dir.as_str());

    let indicator = Indicator::builder(APP_NAME)
        .category(IndicatorCategory::ApplicationStatus)
        .icon_theme_path(icon_path.to_str().unwrap())
        .icon("active-symbolic", "icon")
        .attention_icon("failed-symbolic", "failed")
        .status(IndicatorStatus::Active)
        .title(APP_NAME)
        .label("Display Switch")
        .build();

    let (active_state, failed_state) = current_unit_state();
    let state = ServiceState::new(active_state, failed_state);
    set_icon(&indicator, &state);

    let service_state = Rc::new(RefCell::new(state));

    let m = gtk::Menu::new();

    let mi_enabled = gtk::CheckMenuItem::with_label("Active");
    m.append(&mi_enabled);

    let sep = gtk::SeparatorMenuItem::new();
    m.append(&sep);

    let mi_quit = gtk::MenuItem::with_label("Quit");
    m.append(&mi_quit);
    m.show_all();

    let handler_id: Rc<RefCell<Option<SignalHandlerId>>> = Rc::new(RefCell::new(None));

    let h_id = mi_enabled.connect_activate(glib::clone!(@strong handler_id, @weak indicator, @strong service_state => move |mi_enabled| {
        if mi_enabled.is_active() {
            let mut state = service_state.as_ref().borrow_mut();

            match state.change_state(ActiveStateEnum::Active) {
                FailedStateEnum::Failed => {
                    mi_enabled.block_signal(&handler_id.as_ref().borrow_mut().as_mut().unwrap());
                    mi_enabled.set_active(false);
                    mi_enabled.unblock_signal(&handler_id.as_ref().borrow_mut().as_mut().unwrap());
                }
                _ => (),
            }

            set_icon(&indicator, &state);
        } else {
            let mut state = service_state.as_ref().borrow_mut();

            match state.change_state(ActiveStateEnum::Active) {
                FailedStateEnum::Failed => {
                    mi_enabled.block_signal(&handler_id.as_ref().borrow_mut().as_mut().unwrap());
                    mi_enabled.set_active(false);
                    mi_enabled.unblock_signal(&handler_id.as_ref().borrow_mut().as_mut().unwrap());
                }
                _ => (),
            }

            set_icon(&indicator, &state);
        }
    }));

    *handler_id.as_ref().borrow_mut() = Some(h_id);

    mi_quit.connect_activate(|_| {
        gtk::main_quit();
    });

    indicator.set_menu(Some(&m));

    indicator.set_secondary_activate_target(Some(&mi_enabled));

    gtk::main();
}

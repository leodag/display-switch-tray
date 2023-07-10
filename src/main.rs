use gtk::glib;
use gtk::prelude::*;
use std::process::Command;
use appindicator3::prelude::*;
use appindicator3::{Indicator, IndicatorStatus, IndicatorCategory};

const APP_NAME: &str = "Display Switch Tray";

fn main() {
    gtk::init().unwrap();

    let indicator = Indicator::builder(APP_NAME)
        .category(IndicatorCategory::ApplicationStatus)
        // .menu(&m)
        .icon("computer-symbolic", "icon")
        .status(IndicatorStatus::Passive)
        .title(APP_NAME)
        .label("Display Switch")
        // .secondary_activate_target(&mi_enabled)
        .build();

    let m = gtk::Menu::new();

    let mi_enabled = gtk::CheckMenuItem::with_label("Enabled");
    mi_enabled.set_active(true);
    m.append(&mi_enabled);

    let sep = gtk::SeparatorMenuItem::new();
    m.append(&sep);

    let mi_quit = gtk::MenuItem::with_label("Quit");
    m.append(&mi_quit);
    m.show_all();

    mi_enabled.connect_activate(glib::clone!(@weak indicator => move |mi_enabled| {
        if mi_enabled.is_active() {
            let cmd = Command::new("sudo")
                .args(["systemctl", "start", "display_switch"])
                .status()
                .expect("failed to start");

            indicator.set_icon_full("computer-symbolic", "icon");
            println!("start status {}", cmd.code().unwrap());
        } else {
            let cmd = Command::new("sudo")
                .args(["systemctl", "stop", "display_switch"])
                .status()
                .expect("failed to stop");

            indicator.set_icon_full("action-unavailable-symbolic", "icon");
            println!("stop status {}", cmd.code().unwrap());
        }
    }));

    mi_quit.connect_activate(|_| {
        gtk::main_quit();
    });

    indicator.set_menu(Some(&m));

    indicator.set_secondary_activate_target(Some(&mi_enabled));
    indicator.set_status(IndicatorStatus::Active);

    gtk::main();

    println!("Hello, world!");
}

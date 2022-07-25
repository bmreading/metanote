// app.rs
//
// Copyright 2022 Brian Reading <brian.reading@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::prelude::*;

use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use gtk::gio;
use gtk::gio::{ActionGroup, ActionMap};
use gtk::glib;
use gtk::glib::clone;
use gtk::AboutDialog;
use gtk_macros::action;

use crate::config::{APP_ID, AUTHORS, NAME, VERSION};
use crate::window::MetanoteApplicationWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MetanoteApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for MetanoteApplication {
        const NAME: &'static str = "MetanoteApplication";
        type Type = super::MetanoteApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MetanoteApplication {}
    impl ApplicationImpl for MetanoteApplication {
        fn activate(&self, application: &Self::Type) {
            application.setup_actions();
            let window = MetanoteApplicationWindow::new(application);
            window.present();
        }
    }
    impl GtkApplicationImpl for MetanoteApplication {}
    impl AdwApplicationImpl for MetanoteApplication {}
}

glib::wrapper! {
    pub struct MetanoteApplication(ObjectSubclass<imp::MetanoteApplication>)
    @extends
        gio::Application,
        gtk::Application,
        adw::Application,
    @implements
        ActionGroup,
        ActionMap;
}

impl Default for MetanoteApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl MetanoteApplication {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &APP_ID),
            ("flags", &gio::ApplicationFlags::empty()),
        ])
        .expect("Failed to create MetanoteApplication")
    }

    fn setup_actions(&self) {
        action!(
            self,
            "about",
            clone!(@weak self as app => move |_, _| {
                app.show_about();
            })
        );
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about_dialog = AboutDialog::builder()
            .program_name(NAME)
            .logo_icon_name(APP_ID)
            .version(VERSION)
            .authors(AUTHORS.split(',').map(|a| a.to_string()).collect())
            .modal(true)
            .transient_for(&window)
            .build();

        about_dialog.show();
    }
}

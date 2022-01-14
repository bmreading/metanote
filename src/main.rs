// main.rs
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

mod config;
mod window;

use adw::prelude::*;

use adw::Application;
use gio::{ApplicationFlags, Resource};

use config::{APP_ID, PKGDATADIR};

fn main() {
    // Load and register resources
    let resource = Resource::load(
        format!("{PKGDATADIR}/metanote.gresource"),
    )
    .expect("Could not load resources");
    gio::resources_register(&resource);

    let app = Application::new(Some(APP_ID), ApplicationFlags::empty());

    app.connect_activate(|app| {
        let window = crate::window::MetanoteApplicationWindow::new(app);
        window.present();
    });

    app.run();
}

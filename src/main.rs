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

mod app;
mod config;
mod window;

use adw::prelude::*;

use gio::Resource;

use crate::app::MetanoteApplication;
use crate::config::PKGDATADIR;

fn main() {
    // Load and register resources
    let resource = Resource::load(
        format!("{PKGDATADIR}/metanote.gresource"),
    )
    .expect("Could not load resources");
    gio::resources_register(&resource);

    let app = MetanoteApplication::new();

    let exit_status = app.run();
    std::process::exit(exit_status)
}

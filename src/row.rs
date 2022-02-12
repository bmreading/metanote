// row.rs
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

use adw::Avatar;
use anyhow::{Context, Error, Result};
use gtk::glib;
use gtk::glib::Object;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use crate::metadata::{MetadataAgent, MetadataContainer, MetadataReadCapable};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MetanoteRow {
        // MetadataContainer acts as the main record.
        // All views on this row simply reflect it.
        pub metadata: RefCell<MetadataContainer>,
        pub path: Cell<PathBuf>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MetanoteRow {
        const NAME: &'static str = "MetanoteRow";
        type Type = super::MetanoteRow;
        type ParentType = adw::ActionRow;
    }

    impl ObjectImpl for MetanoteRow {}
    impl WidgetImpl for MetanoteRow {}
    impl ListBoxRowImpl for MetanoteRow {}
    impl PreferencesRowImpl for MetanoteRow {}
    impl ActionRowImpl for MetanoteRow {}
}

gtk::glib::wrapper! {
    pub struct MetanoteRow(ObjectSubclass<imp::MetanoteRow>)
    @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow,gtk::Widget,
        gtk::glib::InitiallyUnowned,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable,
        gtk::ConstraintTarget;
}

impl MetanoteRow {
    pub fn new(path: &Path) -> Result<Self> {
        let metadata = MetadataAgent::default().metadata(path)?;
        let file_name = path
            .file_name()
            .context("{path} is a bad path")?
            .to_owned()
            .into_string()
            .map_err(|_| Error::msg("bad path"))?;

        let row: MetanoteRow = Object::new(&[
            ("title", &Self::title_from_metadata(&metadata)),
            ("subtitle", &file_name),
            ("height-request", &80),
        ])?;

        let avatar;
        if let Some(art) = metadata.art() {
            let cover = art[0].to_picture_widget().paintable().expect("bad art");
            avatar = Avatar::new(50, None, false);
            avatar.set_custom_image(Some(&cover));
            row.add_prefix(&avatar);
        } else {
            avatar = Avatar::new(50, Some(&row.title()), true);
            row.add_prefix(&avatar);
        };

        let imp = row.imp();
        imp.path.set(path.to_path_buf());
        imp.metadata.replace(metadata);

        Ok(row)
    }

    fn title_from_metadata(metadata: &MetadataContainer) -> String {
        let unknown = &"Unknown".to_owned();
        let artist = metadata.artist().as_ref().unwrap_or(unknown);
        let title = metadata.title().as_ref().unwrap_or(unknown);
        format!("{artist} - {title}")
    }

    /// Replaces metadata in row
    pub fn replace_metadata(&self, metadata: MetadataContainer) {
        self.imp().metadata.replace(metadata);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn bad_path_throws_error() {
        let row = MetanoteRow::new(Path::new("bad_path"));
        assert!(row.is_err());
    }
}

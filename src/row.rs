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
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::metadata::{MetadataContainer, MetadataReadCapable, MetadataWriteCapable};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MetanoteRow {
        // MetadataContainer acts as the main record.
        // All views on this row simply reflect it.
        pub metadata: RefCell<MetadataContainer>,
        pub path: RefCell<PathBuf>,
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
    pub fn new<T: MetadataReadCapable>(path: &Path, metadata_agent: &T) -> Result<Self> {
        let metadata = metadata_agent.metadata(path)?;
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
        imp.path.replace(path.to_path_buf());
        imp.metadata.replace(metadata);

        Ok(row)
    }

    fn title_from_metadata(metadata: &MetadataContainer) -> String {
        let unknown = &"Unknown".to_owned();
        let artist = metadata.artist().as_ref().unwrap_or(unknown);
        let title = metadata.title().as_ref().unwrap_or(unknown);
        format!("{artist} - {title}")
    }

    pub fn replace_metadata(&self, metadata: &MetadataContainer) {
        let current = &self.imp().metadata;
        let new = metadata;

        let replacement_metadata = crate::metadata::MetadataContainerBuilder::default()
            .title(self.replace_tag(current.borrow().title(), new.title()))
            .artist(self.replace_tag(current.borrow().artist(), new.artist()))
            .album_artist(self.replace_tag(current.borrow().album_artist(), new.album_artist()))
            .album(self.replace_tag(current.borrow().album(), new.album()))
            .track_number(self.replace_num_tag(current.borrow().track_number(), new.track_number()))
            .track_total(self.replace_num_tag(current.borrow().track_total(), new.track_total()))
            .genre(self.replace_tag(current.borrow().genre(), new.genre()))
            .year(self.replace_tag(current.borrow().year(), new.year()))
            .disc_number(self.replace_num_tag(current.borrow().disc_number(), new.disc_number()))
            .disc_total(self.replace_num_tag(current.borrow().disc_total(), new.disc_total()))
            .composer(self.replace_tag(current.borrow().composer(), new.composer()))
            .comment(self.replace_tag(current.borrow().comment(), new.comment()))
            .copyright(self.replace_tag(current.borrow().copyright(), new.copyright()))
            .art(new.art().to_owned())
            .build()
            .unwrap();

        self.imp().metadata.replace(replacement_metadata);
    }

    fn replace_tag(
        &self,
        current_tag: &Option<String>,
        new_tag: &Option<String>,
    ) -> Option<String> {
        if new_tag == &Some(String::from("<Keep>")) {
            current_tag.to_owned()
        } else {
            new_tag.to_owned()
        }
    }

    fn replace_num_tag(&self, current_tag: &Option<i32>, new_tag: &Option<i32>) -> Option<i32> {
        if new_tag == &Some(-1) {
            current_tag.to_owned()
        } else {
            new_tag.to_owned()
        }
    }

    /// Writes to file whatever metadata that the row holds
    pub fn write_metadata<T: MetadataWriteCapable>(&self, metadata_agent: &T) -> Result<()> {
        let imp = self.imp();
        metadata_agent.write_metadata(&imp.path.borrow(), &imp.metadata.borrow())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::metadata::MetadataAgent;

    #[test]
    fn bad_path_throws_error() {
        let row = MetanoteRow::new(Path::new("bad_path"), &MetadataAgent::new());
        assert!(row.is_err());
    }
}

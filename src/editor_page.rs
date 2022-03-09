// editor_page.rs
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

use adw::{Carousel, PreferencesGroup};
use anyhow::Result;
use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::{Box, CompositeTemplate, Entry, Widget};
use std::cell::RefCell;
use std::ops::Deref;

use crate::art_button::{ArtButton, ArtButtonChangeNotifiable};
use crate::metadata::{Art, MetadataContainer, MetadataWriteCapable};
use crate::row::MetanoteRow;

mod imp {

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/github/bmreading/Metanote/editor_page.ui")]
    pub struct MetanoteEditorPage {
        #[template_child]
        pub art_carousel: TemplateChild<Carousel>,
        #[template_child]
        pub tag_row_group: TemplateChild<PreferencesGroup>,
        #[template_child]
        pub title_text: TemplateChild<Entry>,
        #[template_child]
        pub artist_text: TemplateChild<Entry>,
        #[template_child]
        pub album_artist_text: TemplateChild<Entry>,
        #[template_child]
        pub album_text: TemplateChild<Entry>,
        #[template_child]
        pub track_number_text: TemplateChild<Entry>,
        #[template_child]
        pub track_total_text: TemplateChild<Entry>,
        #[template_child]
        pub genre_text: TemplateChild<Entry>,
        #[template_child]
        pub year_text: TemplateChild<Entry>,
        #[template_child]
        pub disc_number_text: TemplateChild<Entry>,
        #[template_child]
        pub disc_total_text: TemplateChild<Entry>,
        #[template_child]
        pub composer_text: TemplateChild<Entry>,
        #[template_child]
        pub copyright_text: TemplateChild<Entry>,
        #[template_child]
        pub comment_text: TemplateChild<Entry>,

        pub metanote_rows: RefCell<Vec<MetanoteRow>>,
        pub metadata: RefCell<MetadataContainer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MetanoteEditorPage {
        const NAME: &'static str = "MetanoteEditorPage";
        type Type = super::MetanoteEditorPage;
        type ParentType = Box;

        fn new() -> Self {
            Self {
                art_carousel: TemplateChild::default(),
                tag_row_group: TemplateChild::default(),
                title_text: TemplateChild::default(),
                artist_text: TemplateChild::default(),
                album_artist_text: TemplateChild::default(),
                album_text: TemplateChild::default(),
                track_number_text: TemplateChild::default(),
                track_total_text: TemplateChild::default(),
                genre_text: TemplateChild::default(),
                year_text: TemplateChild::default(),
                disc_number_text: TemplateChild::default(),
                disc_total_text: TemplateChild::default(),
                composer_text: TemplateChild::default(),
                copyright_text: TemplateChild::default(),
                comment_text: TemplateChild::default(),
                metanote_rows: Default::default(),
                metadata: Default::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MetanoteEditorPage {}
    impl WidgetImpl for MetanoteEditorPage {}
    impl BoxImpl for MetanoteEditorPage {}
}

glib::wrapper! {
    pub struct MetanoteEditorPage(ObjectSubclass<imp::MetanoteEditorPage>)
        @extends
            Box,
            Widget,
        @implements
            gtk::Accessible,
            gtk::Buildable,
            gtk::ConstraintTarget,
            gtk::Orientable;
}

impl MetanoteEditorPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("failed to create MetanoteEditorPage")
    }

    pub fn set_metadata(&self, rows: &[MetanoteRow]) {
        let imp = self.imp();
        imp.metanote_rows.replace(rows.to_vec());

        if rows.len() > 0 {
            // Consolidate metadata into vec
            let mut metadata_containers = Vec::new();
            for row in imp.metanote_rows.borrow().iter() {
                metadata_containers.push(row.imp().metadata.borrow().clone());
            }

            // Merge like metadata fields to determine appropriate metadata to operate on
            let merged = MetadataContainer::merge(&metadata_containers);
            self.set_artwork(&merged);
            self.set_textual_tags(&merged);
            imp.metadata.replace(merged);
        }
    }

    fn set_artwork(&self, metadata: &MetadataContainer) {
        self.clear_art_carousel();
        if let Some(art) = metadata.art() {
            for art_element in art {
                self.imp()
                    .art_carousel
                    .append(&ArtButton::with_art(&art_element, Some(self)));
            }
        } else {
            self.imp().art_carousel.append(&ArtButton::new(Some(self)));
        }
    }

    fn clear_art_carousel(&self) {
        let imp = self.imp();
        let children = imp.art_carousel.observe_children();
        for child in children {
            let child_widget = child.downcast::<gtk::Widget>().unwrap();
            imp.art_carousel.remove(&child_widget);
        }
    }

    fn set_textual_tags(&self, metadata: &MetadataContainer) {
        let imp = self.imp();

        let tags = [
            (&imp.title_text, EntryValue::Text(metadata.title())),
            (&imp.artist_text, EntryValue::Text(metadata.artist())),
            (&imp.album_text, EntryValue::Text(metadata.album())),
            (
                &imp.album_artist_text,
                EntryValue::Text(metadata.album_artist()),
            ),
            (
                &imp.track_number_text,
                EntryValue::Number(metadata.track_number()),
            ),
            (
                &imp.track_total_text,
                EntryValue::Number(metadata.track_total()),
            ),
            (&imp.genre_text, EntryValue::Text(metadata.genre())),
            (&imp.year_text, EntryValue::Text(metadata.year())),
            (
                &imp.disc_number_text,
                EntryValue::Number(metadata.disc_number()),
            ),
            (
                &imp.disc_total_text,
                EntryValue::Number(metadata.disc_total()),
            ),
            (&imp.composer_text, EntryValue::Text(metadata.composer())),
            (&imp.copyright_text, EntryValue::Text(metadata.copyright())),
            (&imp.comment_text, EntryValue::Text(metadata.comment())),
        ];

        for tag in tags {
            self.set_text_value(tag.0, tag.1);
        }
    }

    fn set_text_value(&self, entry: &TemplateChild<Entry>, entry_value: EntryValue) {
        entry.set_placeholder_text(None);
        
        match entry_value {
            EntryValue::Text(v) => {
                match v {
                    Some(v) => {
                        if v == "<Keep>" {
                            entry.set_placeholder_text(Some(v));
                            entry.set_text("");
                        } else {
                            entry.set_text(v);
                        }
                    }
                    None => { entry.set_text("") },
                }
            }
            EntryValue::Number(v) => {
                match v {
                    Some(v) => {
                        let v = v.to_string();
                        if v == "-1" {
                            entry.set_text("");
                            entry.set_placeholder_text(Some("<Keep>"));
                        } else {
                            entry.set_text(&v);
                        }
                    }
                    None => { entry.set_text("") }
                }
            }
        }
    }

    /// Writes metadata to whichever tracks editor has
    pub fn write_metadata<T: MetadataWriteCapable>(&self, metadata_agent: &T) -> Result<()> {
        self.update_metadata();
        let imp = self.imp();
        let tracks = imp.metanote_rows.borrow();
        let current_metadata = imp.metadata.borrow();

        for track in tracks.iter() {
            track.replace_metadata(&current_metadata);
            match track.write_metadata(metadata_agent) {
                Ok(_) => (),
                Err(e) => log::error!(
                    "failed to write metadata for file at path {} - {}",
                    track.imp().path.borrow().to_str().unwrap(),
                    e
                ),
            }
        }
        Ok(())
    }


    // Replace instance's metadata with what has been modified in UI
    fn update_metadata(&self) {

        let updated_metadata = crate::metadata::MetadataContainerBuilder::default()
            .title(StringOption::from(&self.imp().title_text).0)
            .artist(StringOption::from(&self.imp().artist_text).0)
            .album_artist(StringOption::from(&self.imp().album_artist_text).0)
            .album(StringOption::from(&self.imp().album_text).0)
            .track_number(I32Option::from(&self.imp().track_number_text).0)
            .track_total(I32Option::from(&self.imp().track_total_text).0)
            .genre(StringOption::from(&self.imp().genre_text).0)
            .year(StringOption::from(&self.imp().year_text).0)
            .disc_number(I32Option::from(&self.imp().disc_number_text).0)
            .disc_total(I32Option::from(&self.imp().disc_total_text).0)
            .composer(StringOption::from(&self.imp().composer_text).0)
            .comment(StringOption::from(&self.imp().comment_text).0)
            .copyright(StringOption::from(&self.imp().copyright_text).0)
            .art(self.imp().metadata.take().art().to_owned())
            .build()
            .unwrap();

        self.imp().metadata.replace(updated_metadata);
    }
}

// Struct to wrap an Option<String> so we can add traits to it
struct StringOption(Option<String>);
impl From<&TemplateChild<Entry>> for StringOption {
    fn from(entry: &TemplateChild<Entry>) -> Self {
        if entry.placeholder_text().is_some() && entry.text().len() == 0 {
            StringOption(Some(entry.placeholder_text().unwrap().to_string()))
        } else if entry.text().len() > 0 {
            StringOption(Some(entry.text().to_string()))
        } else {
            StringOption(None)
        }
    }
}

// Struct to wrap an Option<i32> so we can add traits to it
struct I32Option(Option<i32>);
impl From<&TemplateChild<Entry>> for I32Option {
    fn from(entry: &TemplateChild<Entry>) -> Self {
        if entry.placeholder_text().is_some() && entry.text().len() == 0 {
            I32Option(Some(-1))
        } else if entry.text().len() > 0 {
            I32Option(Some(entry.text().parse::<i32>().expect("failed to parse i32")))
        } else {
            I32Option(None)
        }
    }
}

impl ArtButtonChangeNotifiable for MetanoteEditorPage {
    fn on_art_change(&self) {
        let art_carousel = &self.imp().art_carousel;
        let mut artwork = Vec::new();

        for i in 0..art_carousel.n_pages() {
            let art_button = art_carousel.nth_page(i).downcast::<ArtButton>().unwrap();

            match &art_button.imp().path.borrow().deref() {
                Some(art_path) => {
                    let art = Art::from_path(art_path).unwrap();
                    artwork.push(art);
                }
                None => {
                    artwork.pop();
                }
            };
        }
        self.imp().metadata.borrow_mut().set_art(Some(artwork));
    }
}

enum EntryValue<'a> {
    Text(&'a Option<String>),
    Number(&'a Option<i32>),
}

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
use gtk::glib::clone;
use gtk::glib::subclass::InitializingObject;
use gtk::{Box, CompositeTemplate, Entry, Widget};
use std::cell::RefCell;

use crate::metadata::{MetadataContainer, MetadataWriteCapable};
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
        pub genre_text: TemplateChild<Entry>,
        #[template_child]
        pub year_text: TemplateChild<Entry>,

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
                genre_text: TemplateChild::default(),
                year_text: TemplateChild::default(),
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

    impl ObjectImpl for MetanoteEditorPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_callbacks();
        }
    }

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

    fn setup_callbacks(&self) {
        self.imp()
            .title_text
            .connect_changed(clone!(@weak self as page => move |title| {
                let mut current_metadata = page.imp().metadata.borrow_mut();
                current_metadata.set_title(Some(title.text().to_string()));
            }));

        self.imp()
            .artist_text
            .connect_changed(clone!(@weak self as page => move |artist| {
                let mut current_metadata = page.imp().metadata.borrow_mut();
                current_metadata.set_artist(Some(artist.text().to_string()));
            }));

        self.imp().album_artist_text.connect_changed(
            clone!(@weak self as page => move |album_artist| {
                let mut current_metadata = page.imp().metadata.borrow_mut();
                current_metadata.set_album_artist(Some(album_artist.text().to_string()));
            }),
        );

        self.imp()
            .album_text
            .connect_changed(clone!(@weak self as page => move |album| {
                let mut current_metadata = page.imp().metadata.borrow_mut();
                current_metadata.set_album(Some(album.text().to_string()));
                log::debug!("{:?}", current_metadata);
            }));

        self.imp()
            .year_text
            .connect_changed(clone!(@weak self as page => move |year| {
                let mut current_metadata = page.imp().metadata.borrow_mut();
                current_metadata.set_year(Some(year.text().to_string()));
            }));
    }

    fn set_artwork(&self, metadata: &MetadataContainer) {
        self.clear_art_carousel();
        if let Some(art) = metadata.art() {
            for art_element in art {
                self.imp()
                    .art_carousel
                    .append(&art_element.to_picture_widget());
            }
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
        let empty_value = String::from("");
        imp.title_text
            .set_text(metadata.title().as_ref().unwrap_or(&empty_value));
        imp.artist_text
            .set_text(metadata.artist().as_ref().unwrap_or(&empty_value));
        imp.album_artist_text
            .set_text(metadata.album_artist().as_ref().unwrap_or(&empty_value));
        imp.album_text
            .set_text(metadata.album().as_ref().unwrap_or(&empty_value));
        imp.year_text
            .set_text(metadata.year().as_ref().unwrap_or(&empty_value));
    }

    /// Writes metadata to whichever tracks editor has
    pub fn write_metadata<T: MetadataWriteCapable>(&self, metadata_agent: &T) -> Result<()> {
        let imp = self.imp();
        let tracks = imp.metanote_rows.borrow();
        let metadata_to_write = imp.metadata.borrow();

        for track in tracks.iter() {
            track.replace_metadata(&metadata_to_write);
            match track.write_metadata(metadata_agent) {
                Ok(_) => todo!(),
                Err(e) => log::error!(
                    "failed to write metadata for file at path {} - {}",
                    track.imp().path.borrow().to_str().unwrap(),
                    e
                ),
            }
        }
        Ok(())
    }
}

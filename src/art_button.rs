// art_button.rs
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

use gtk::prelude::*;

use gtk::subclass::prelude::*;

use gtk::glib;
use gtk::glib::{clone, Object};
use gtk::{Button, FileChooserAction, FileChooserNative, ResponseType};

use std::path::PathBuf;
use std::cell::RefCell;

use crate::metadata::Art;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ArtButton {
        pub file_chooser: FileChooserNative,
        pub path: RefCell<PathBuf>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArtButton {
        const NAME: &'static str = "ArtButton";
        type Type = super::ArtButton;
        type ParentType = Button;

        fn new() -> Self {
            let file_chooser = FileChooserNative::builder()
                .modal(true)
                .action(FileChooserAction::Open)
                .build();

            Self { 
                file_chooser,
                path: Default::default(),
            }
        }
    }

    impl ObjectImpl for ArtButton {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_callbacks();
        }
    }
    impl WidgetImpl for ArtButton {}
    impl ButtonImpl for ArtButton {}
}

glib::wrapper! {
    pub struct ArtButton(ObjectSubclass<imp::ArtButton>)
        @extends
            gtk::Button,
            gtk::Widget,
            glib::InitiallyUnowned,
        @implements
            gtk::Accessible,
            gtk::Actionable,
            gtk::Buildable,
            gtk::ConstraintTarget;
}

impl ArtButton {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create `ArtButton`.")
    }

    pub fn with_art(art: &Art) -> Self {
        let picture = art.to_picture_widget();

        Object::new(&[("child", &picture)]).expect("Failed to create `ArtButton`.")
    }

    fn setup_callbacks(&self) {
        self.connect_clicked(move |button| {
            button.imp().file_chooser.show();
        });

        self.imp().file_chooser.connect_response(
            clone!(@weak self as button => move |fc, response| {
                if response == ResponseType::Accept {
                    let path = &fc.file().unwrap().path().unwrap();
                    let art = Art::from_path(path).unwrap();
                    button.imp().path.replace(path.to_path_buf());
                    button.set_child(Some(&art.to_picture_widget()));
                }
            }),
        );
    }
}

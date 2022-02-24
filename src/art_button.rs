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
use gtk::glib::{clone, Object, ToValue, Value};
use gtk::{Button, FileChooserAction, FileChooserNative, ResponseType};

use std::cell::RefCell;
use std::ops::Deref;
use std::path::PathBuf;

use crate::editor_page::MetanoteEditorPage;
use crate::metadata::Art;

mod imp {
    use super::*;

    pub struct ArtButton {
        pub file_chooser: FileChooserNative,
        pub path: RefCell<Option<PathBuf>>,
        pub notifiable: RefCell<Option<Value>>,
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
                .title("Choose an image")
                .build();

            Self {
                file_chooser,
                path: Default::default(),
                notifiable: RefCell::new(None),
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
    pub fn new<T>(notifiable: Option<&T>) -> Self
    where
        T: ArtButtonChangeNotifiable + ToValue,
    {
        if let Some(notifiable) = notifiable {
            let button: ArtButton =
                Object::new(&[("label", &"Click to set artwork"), ("hexpand", &true)])
                    .expect("failed to create `ArtButton`.");

            button
                .imp()
                .notifiable
                .replace(Some(Value::from(notifiable)));

            button
        } else {
            Default::default()
        }
    }

    pub fn with_art<T>(art: &Art, notifiable: Option<&T>) -> Self
    where
        T: ArtButtonChangeNotifiable + ToValue,
    {
        let picture = art.to_picture_widget();
        let button: ArtButton =
            Object::new(&[("child", &picture)]).expect("failed to create `ArtButton`.");

        if let Some(notifiable) = notifiable {
            button
                .imp()
                .notifiable
                .replace(Some(Value::from(notifiable)));
        }

        button
    }

    fn setup_callbacks(&self) {
        self.connect_clicked(move |button| {
            let window = button
                .root()
                .expect("failed to get art button's root")
                .downcast::<gtk::ApplicationWindow>()
                .expect("failed to get art button's application window");
            button.imp().file_chooser.set_transient_for(Some(&window));
            button.imp().file_chooser.show();
        });

        self.imp().file_chooser.connect_response(
            clone!(@weak self as button => move |fc, response| {
                if response == ResponseType::Accept {
                    let path = &fc.file().unwrap().path().unwrap();
                    let art = Art::from_path(path).unwrap();
                    button.imp().path.replace(Some(path.to_path_buf()));
                    button.set_child(Some(&art.to_picture_widget()));

                    if let Some(notifiable) =  button.imp().notifiable.borrow().deref() {
                        // Due to limitations of Value, a concrete type must be specified
                        // so if expected notifiable type changes, this line must also change
                        let notifiable = notifiable.get::<MetanoteEditorPage>().expect("failed to get editor page");

                        notifiable.on_art_change();
                    }
                }
            }),
        );
    }
}

impl Default for ArtButton {
    fn default() -> Self {
        Object::new(&[]).expect("failed to create `ArtButton`.")
    }
}

/// A trait that gives a type the
/// ability to perform a callback
/// function when an ArtButton's
/// art changes
pub trait ArtButtonChangeNotifiable {
    fn on_art_change(&self);
}

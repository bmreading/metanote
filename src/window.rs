// window.rs
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

use adw::WindowTitle;
use anyhow::Result;
use gtk::gio;
use gtk::gio::{File, FileInfo};
use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::glib::{clone, Object};
use gtk::{
    Button, CompositeTemplate, FileChooserAction, FileChooserNative, ListBox, ResponseType, Stack,
};
use gtk_macros::action;

use crate::app::MetanoteApplication;
use crate::editor_page::MetanoteEditorPage;
use crate::metadata::MetadataAgent;
use crate::row::MetanoteRow;

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/github/bmreading/Metanote/window.ui")]
    pub struct MetanoteApplicationWindow {
        pub file_chooser: FileChooserNative,

        #[template_child]
        pub content_stack: TemplateChild<Stack>,
        #[template_child]
        pub tracklist: TemplateChild<ListBox>,
        #[template_child]
        pub main_title: TemplateChild<WindowTitle>,
        #[template_child]
        pub save_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MetanoteApplicationWindow {
        const NAME: &'static str = "MetanoteApplicationWindow";
        type Type = super::MetanoteApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            // Since FileChooserNative requires a reference to outlive
            // its invocation closure, we will instantiate it here, and
            // simply call it later
            let file_chooser = FileChooserNative::builder()
                .modal(true)
                .action(FileChooserAction::SelectFolder)
                .build();

            Self {
                file_chooser,
                content_stack: TemplateChild::default(),
                tracklist: TemplateChild::default(),
                main_title: TemplateChild::default(),
                save_button: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MetanoteApplicationWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.bind_editor_page();
            obj.setup_actions();
            obj.setup_callbacks();
        }
    }

    impl WidgetImpl for MetanoteApplicationWindow {}
    impl WindowImpl for MetanoteApplicationWindow {}
    impl ApplicationWindowImpl for MetanoteApplicationWindow {}
    impl AdwApplicationWindowImpl for MetanoteApplicationWindow {}
}

glib::wrapper! {
    pub struct MetanoteApplicationWindow(ObjectSubclass<imp::MetanoteApplicationWindow>)
    @extends
        adw::ApplicationWindow,
        gtk::ApplicationWindow,
        gtk::Window,
        gtk::Widget,
    @implements 
        gio::ActionGroup,
        gio::ActionMap,
        gtk::Accessible,
        gtk::Buildable,
        gtk::ConstraintTarget,
        gtk::Native,
        gtk::Root,
        gtk::ShortcutManager;
}

impl MetanoteApplicationWindow {
    pub fn new(app: &MetanoteApplication) -> Self {
        Object::new(&[("application", app)]).expect("Failed to create Metanote window")
    }

    fn setup_actions(&self) {
        action!(
            self,
            "open",
            clone!(@weak self as window => move |_, _| {
                let file_chooser = &window.imp().file_chooser;
                file_chooser.set_transient_for(Some(&window));

                file_chooser.connect_response(clone!(@weak window => move |fc, response| {
                    if response == ResponseType::Accept {
                        window.add_tracks(&fc.file().expect("Could not retrieve folder from file chooser"));
                    }
                    fc.destroy();
                }));

                file_chooser.show();
            })
        );
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();

        imp.tracklist
            .connect_selected_rows_changed(clone!(@weak self as window => move |tracklist| {
                let content_stack = &window.imp().content_stack;

                if tracklist.selected_rows().len() > 0 {
                    let mut selected_tracks = Vec::new();
                    for track in tracklist.selected_rows() {
                        let track = track.downcast::<MetanoteRow>().unwrap();
                        selected_tracks.push(track);
                    }

                    let editor_page = content_stack.child_by_name("editor_page").unwrap().downcast::<MetanoteEditorPage>().unwrap();
                    editor_page.set_metadata(&selected_tracks);
                    content_stack.set_visible_child(&editor_page);
                } else {
                    content_stack.set_visible_child_name("status_page");
                }
        }));

        imp.save_button
            .connect_clicked(clone!(@weak self as window => move |_| {
                let editor_page = window.imp().content_stack.child_by_name("editor_page").unwrap().downcast::<MetanoteEditorPage>().unwrap();
                let agent = MetadataAgent::new();
                match editor_page.write_metadata(&agent) {
                    Ok(_) => (),
                    Err(e) => log::error!("Failed to save tracks, {}", e),
                };
        }));
    }

    fn add_tracks(&self, dir: &File) {
        let tracklist = &self.imp().tracklist;
        self.clear_tracklist();

        let tracks = self.parse_dir(dir).expect("couldn't parse tracks");

        for track in tracks {
            let path = dir.path().unwrap().join(track.name().as_path());

            let agent = MetadataAgent::new();
            let row = crate::row::MetanoteRow::new(&path, &agent);
            match row {
                Ok(row) => tracklist.append(&row),
                Err(err) => log::warn!("unable to display track, {err}"),
            }
        }
    }

    fn clear_tracklist(&self) {
        let tracklist = &self.imp().tracklist;
        tracklist.select_all();

        for track in tracklist.selected_rows() {
            tracklist.remove(&track);
        }
    }

    fn parse_dir(&self, dir: &File) -> Result<Vec<FileInfo>> {
        let file_enumerator = dir.enumerate_children(
            "*",
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            Some(&gio::Cancellable::new()),
        )?;

        let mut audio_tracks = Vec::new();
        for child in file_enumerator {
            let file_info = child?;
            if file_info.content_type().unwrap().contains("audio") {
                audio_tracks.push(file_info);
            }
        }

        Ok(audio_tracks)
    }

    fn bind_editor_page(&self) {
        let content_stack = &self.imp().content_stack;

        // Hack to add a name to GtkStackPage added from composite template
        let status_page = content_stack.visible_child().unwrap();
        content_stack.remove(&status_page);
        content_stack.add_named(&status_page, Some("status_page"));

        let editor_page = MetanoteEditorPage::new();
        content_stack.add_named(&editor_page, Some("editor_page"));
    }
}

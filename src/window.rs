use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use adw::WindowTitle;
use anyhow::Result;
use glib::{clone, Object};
use glib::subclass::InitializingObject;
use gio::{File, FileInfo};
use gtk::{CompositeTemplate, FileChooserAction, FileChooserNative, ListBox, ResponseType};
use gtk_macros::action;
use log::warn;

use crate::app::MetanoteApplication;

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/github/bmreading/Metanote/window.ui")]
    pub struct MetanoteApplicationWindow {
        pub file_chooser: FileChooserNative,

        #[template_child]
        pub tracklist: TemplateChild<ListBox>,
        #[template_child]
        pub main_title: TemplateChild<WindowTitle>,
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
                tracklist: TemplateChild::default(),
                main_title: TemplateChild::default(),
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
            obj.setup_actions();
        }
    }
    
    impl WidgetImpl for MetanoteApplicationWindow {}
    impl WindowImpl for MetanoteApplicationWindow {}
    impl ApplicationWindowImpl for MetanoteApplicationWindow {}
    impl AdwApplicationWindowImpl for MetanoteApplicationWindow {}
}

glib::wrapper! {
    pub struct MetanoteApplicationWindow(ObjectSubclass<imp::MetanoteApplicationWindow>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
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
                        window.add_tracks(fc.file().expect("Could not retrieve folder from file chooser"));
                    }
                    fc.destroy();
                }));

                file_chooser.show();
            })
        );
    }

    fn add_tracks(&self, dir: File) {
        let tracklist = &self.imp().tracklist;
        self.clear_tracklist();

        match self.parse_dir(dir) {
            Ok(tracks) => {
                for track in tracks {
                    let row = adw::ActionRow::builder()
                        .title("Unknown Artist - Unknown Album")
                        .subtitle(track.name().to_str().unwrap())
                        .build();
                
                    tracklist.append(&row);
                }
            }
            Err(err) => warn!("unable to parse directory, {err}")
        }
    }

    fn clear_tracklist(&self) {
        let tracklist = &self.imp().tracklist;
        tracklist.select_all();
        
        for track in tracklist.selected_rows() {
            tracklist.remove(&track);
        }
    }

    fn parse_dir(&self, dir: File) ->  Result<Vec<FileInfo>>{
        let file_enumerator = dir.enumerate_children(
            "*", 
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS, 
            Some(&gio::Cancellable::new()))?;

        let mut audio_tracks = Vec::new();
        for child in file_enumerator {
            let file_info = child?;
            if file_info.content_type().unwrap().contains("audio") {
                audio_tracks.push(file_info);
            }            
        }

        Ok(audio_tracks)
    }
}

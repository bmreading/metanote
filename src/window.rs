use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use adw::WindowTitle;
use glib::subclass::InitializingObject;
use glib::Object;
use gtk::{CompositeTemplate, ListBox};

use crate::app::MetanoteApplication;

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/github/bmreading/Metanote/window.ui")]
    pub struct MetanoteApplicationWindow {
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

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MetanoteApplicationWindow {}
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
}

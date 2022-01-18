use adw::prelude::*;

use adw::subclass::prelude::*;
use gtk::subclass::prelude::*;

use gio::{ActionGroup, ActionMap};
use glib::object_subclass;

use crate::config::APP_ID;
use crate::window::MetanoteApplicationWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MetanoteApplication;

    #[object_subclass]
    impl ObjectSubclass for MetanoteApplication {
        const NAME: &'static str = "MetanoteApplication";
        type Type = super::MetanoteApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MetanoteApplication {}
    impl ApplicationImpl for MetanoteApplication {
        fn activate(&self, application: &Self::Type) {
            let window = MetanoteApplicationWindow::new(application);
            window.present();
        }
    }
    impl GtkApplicationImpl for MetanoteApplication {}
    impl AdwApplicationImpl for MetanoteApplication {}
}

glib::wrapper! {
    pub struct MetanoteApplication(ObjectSubclass<imp::MetanoteApplication>)
    @extends gio::Application, gtk::Application, adw::Application,
    @implements ActionGroup, ActionMap;
}

impl Default for MetanoteApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl MetanoteApplication {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &APP_ID),
            ("flags", &gio::ApplicationFlags::empty()),
        ])
        .expect("Failed to create MetanoteApplication")
    }
}

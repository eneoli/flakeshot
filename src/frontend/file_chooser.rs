use std::path::PathBuf;

use gtk::prelude::*;
use gtk4_layer_shell::Layer;
use gtk4_layer_shell::LayerShell;
use relm4::prelude::*;
use relm4::SimpleComponent;

use crate::backend::is_wayland;

pub struct FileChooserInit {
    pub on_submit: Box<dyn Fn(Option<PathBuf>)>,
}

#[derive(Debug)]
pub enum FileChooserEvent {
    Cancel,
    Save(PathBuf),
}

pub struct FileChooser {
    on_submit: Box<dyn Fn(Option<PathBuf>)>,
}

impl FileChooser {
    pub fn open<F>(on_submit: F)
    where
        F: Fn(Option<PathBuf>) + 'static,
    {
        let mut file_chooser = FileChooser::builder().launch(FileChooserInit {
            on_submit: Box::new(on_submit),
        });

        file_chooser.widget().show();
        file_chooser.detach_runtime();
    }
}

#[relm4::component(pub)]
impl SimpleComponent for FileChooser {
    type Init = FileChooserInit;
    type Input = FileChooserEvent;
    type Output = ();

    view! {
        root = gtk::Window {
            set_keyboard_mode: gtk4_layer_shell::KeyboardMode::OnDemand,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                gtk::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &gtk::Label {
                       set_label: "Save file to file system",
                    },

                    set_show_title_buttons: false,

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        set_width_request: 100,
                        set_height_request: 25,
                        connect_clicked[sender, root] => move |_| {
                            root.destroy();
                            sender.input(FileChooserEvent::Cancel);
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Save",
                        set_width_request: 100,
                        set_height_request: 25,
                        connect_clicked[sender, file_chooser, root] => move |_| {
                            let file = file_chooser
                            .file()
                            .map(|f| {
                                f.path()
                                 .expect("FileChooser didn't return a path?")
                            });

                            if let Some(path) = file {
                                root.destroy();
                                sender.input(FileChooserEvent::Save(path));
                            }

                        },
                    },
                },

                #[name = "file_chooser"]
                gtk::FileChooserWidget {
                    set_action: gtk::FileChooserAction::Save,
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        if is_wayland() {
            root.init_layer_shell();
            root.set_layer(Layer::Overlay);
        }

        let widgets = view_output!();

        let model = FileChooser {
            on_submit: init.on_submit,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            FileChooserEvent::Cancel => {
                (self.on_submit)(None);
            }
            FileChooserEvent::Save(file) => {
                (self.on_submit)(Some(file));
            }
        }
    }
}

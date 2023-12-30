use gtk::prelude::*;
use gtk4_layer_shell::Layer;
use gtk4_layer_shell::LayerShell;
use relm4::prelude::*;
use relm4::SimpleComponent;

pub struct FileChooserInit {
    pub on_submit: fn(Option<String>) -> (),
}

#[derive(Debug)]
pub enum FileChooserEvent {
    Cancel,
    Save(String),
}

#[derive(Debug)]
pub struct FileChooserModel {
    on_submit: fn(Option<String>) -> (),
}

#[relm4::component(pub)]
impl SimpleComponent for FileChooserModel {
    type Init = FileChooserInit;
    type Input = FileChooserEvent;
    type Output = ();

    view! {
        root = gtk::Window {
            init_layer_shell: (),
            set_layer: Layer::Overlay,
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
                                 .expect("FileChooser returned invalid file?")
                                 .into_os_string()
                                 .into_string()
                                 .expect("Path is not UTF-8 encoded.")
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
        let widgets = view_output!();

        let model = FileChooserModel {
            on_submit: init.on_submit,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
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

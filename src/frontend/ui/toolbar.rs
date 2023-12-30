use gtk::prelude::*;
use relm4::gtk::Align;
use relm4::prelude::*;
use relm4::SimpleComponent;

#[derive(Debug)]
pub struct Toolbar {}

#[derive(Debug)]
pub enum ToolbarEvent {
    SaveAsFile,
    SaveIntoClipboard,
}

#[relm4::component(pub)]
impl SimpleComponent for Toolbar {
    type Input = ();
    type Output = ToolbarEvent;
    type Init = ();

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_hexpand: false,
            set_vexpand: false,
            set_valign: Align::End,
            set_halign: Align::Center,
            add_css_class: "toolbar",
            gtk::Button {
                set_icon_name: "paper",
                set_tooltip_text: Some("Save to file"),
                add_css_class: "toolbar-button",
                connect_clicked[sender] => move |_| {
                    sender.output(ToolbarEvent::SaveAsFile).unwrap();
                },
            },

            gtk::Button {
                set_icon_name: "copy",
                add_css_class: "toolbar-button",
                set_tooltip_text: Some("Copy to clipboard (TODO)"),
                connect_clicked[sender] => move |_| {
                    sender.output(ToolbarEvent::SaveIntoClipboard).unwrap();
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let model = Toolbar {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

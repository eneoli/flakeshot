struct Tray {}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "Flakeshot Tray".into()
    }

    fn title(&self) -> String {
        "Your mom".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let icon_pixmap = {
            let image = image::io::Reader::open("./arch_grub.jpg")
                .unwrap()
                .decode()
                .unwrap();
            let width = image.width() as i32;
            let height = image.height() as i32;
            let data = image.into_rgba8().into_raw();

            let icon = ksni::Icon {
                width,
                height,
                data,
            };

            vec![icon]
        };

        ksni::ToolTip {
            icon_name: "Flakeshot".into(),
            icon_pixmap,
            title: "Your mom".into(),
            description: "I use Arch btw.".into(),
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        vec![SubMenu {
            label: "Kekw".into(),
            ..Default::default()
        }
        .into()]
    }
}

pub fn start() {
    let _handle = ksni::spawn(Tray {}).unwrap();

    loop {
        std::thread::park();
    }
}

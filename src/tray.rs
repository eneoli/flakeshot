use tray_icon::TrayIconBuilder;

pub fn start() {
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("I use Arch btw.")
        .build()
        .unwrap();

    loop {}
}

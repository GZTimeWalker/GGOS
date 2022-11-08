use super::gop::GOPDisplay;

once_mutex!(pub DISPLAY: GOPDisplay<'static>);

pub fn init(boot_info: &'static boot::BootInfo) {
    let graphic = &boot_info.graphic_info;
    init_DISPLAY(GOPDisplay::new(graphic));

    let mut display = get_display_for_sure();

    display.clear(None, 0);
    let (x, y) = display.resolution();

    info!("Display: {}x{}", x, y);
    info!("VGA Display Initialized.");
}

guard_access_fn! {
    pub get_display(DISPLAY: GOPDisplay<'static>)
}

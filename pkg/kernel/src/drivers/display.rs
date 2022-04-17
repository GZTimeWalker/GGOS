use crate::utils::GOPDisplay;

once_mutex!(pub DISPLAY: GOPDisplay<'static>);

pub fn init(boot_info: &'static boot::BootInfo) {
    let graphic = &boot_info.graphic_info;
    init_DISPLAY(GOPDisplay::new(graphic));

    get_display_for_sure().clear(None, 0);
    info!("VGA Display Initialized.");
}

guard_access_fn! {
    pub get_display(DISPLAY: GOPDisplay<'static>)
}

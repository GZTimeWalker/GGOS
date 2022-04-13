use crate::utils::GOPDisplay;
use boot::GraphicInfo;

once_mutex!(pub DISPLAY: GOPDisplay<'static>);

pub fn init(graphic: &'static GraphicInfo) {
    init_DISPLAY(GOPDisplay::new(graphic));
}

guard_access_fn! {
    pub get_display(DISPLAY: GOPDisplay<'static>)
}

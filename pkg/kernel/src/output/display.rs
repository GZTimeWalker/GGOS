use crate::utils::GOPDisplay;
use boot::GraphicInfo;

once_mutex!(pub DISPLAY: GOPDisplay<'static>);

pub fn initialize(graphic: &'static GraphicInfo) {
    init_DISPLAY(GOPDisplay::new(graphic));
}

guard_access_fn! {
    #[doc = "基于GOP的显示器"]
    pub get_display(DISPLAY: GOPDisplay<'static>)
}

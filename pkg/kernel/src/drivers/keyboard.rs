use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1, layouts};
pub type DefaultKeyBoard = Keyboard<layouts::Us104Key, ScancodeSet1>;

once_mutex!(pub KEYBOARD: DefaultKeyBoard);

guard_access_fn!(pub get_keyboard(KEYBOARD: DefaultKeyBoard));

pub fn init() {
    init_KEYBOARD(Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    ));
    info!("Keyboard Initialized.");
}

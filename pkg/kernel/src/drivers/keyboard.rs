use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
pub type DefaultKeyBoard = Keyboard<layouts::Us104Key, ScancodeSet1>;

once_mutex!(pub KEYBOARD: DefaultKeyBoard);

guard_access_fn!(pub get_keyboard(KEYBOARD: DefaultKeyBoard));

pub fn init() {
    init_KEYBOARD(Keyboard::new(HandleControl::Ignore));
    info!("Keyboard Initialized.");
}

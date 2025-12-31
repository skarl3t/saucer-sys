use saucer_sys::*;
use std::ptr::null_mut;

fn main() {
    unsafe {
        if std::env::var("THIS_WILL_NOT_HAPPEN").is_err() {
            return;
        }

        let app = saucer_application_new(null_mut(), null_mut());
        saucer_window_new(null_mut(), null_mut());
        saucer_desktop_new(app);
    }
}

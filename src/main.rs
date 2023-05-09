use xcb::{x, Xid};

mod input;
mod capture;
mod network;
fn main() -> () {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let root_window = screen.root();

    let grab_mode = x::GrabMode::Async;

    let cursor = x::Cursor::none();

    let request = xcb::x::GrabPointer{
        owner_events: false,
        grab_window: root_window,
        event_mask: x::EventMask::BUTTON_PRESS | x::EventMask::BUTTON_RELEASE | x::EventMask::POINTER_MOTION,
        pointer_mode: grab_mode,
        keyboard_mode: grab_mode,
        confine_to: x::Window::none(),
        cursor,
        time: x::CURRENT_TIME,
    };

    let cookie = conn.send_request(&request);
    let reply = conn.wait_for_reply(cookie);

    if reply.is_err() {
        println!("Failed to grab pointer");
        return;
    }

    loop {
        let event = conn.wait_for_event();

        if let Ok(e) = event {
            match e {
                xcb::Event::X(x::Event::ButtonPress(mv)) => {
                    println!("Button press");
                }
                xcb::Event::X(x::Event::ButtonRelease(mv)) => {
                    println!("Button release");
                }
                xcb::Event::X(x::Event::MotionNotify(mv)) => {
                    println!("Motion notify");
                }
                _ => {}
            }
        }
    }

    x::UngrabPointer{time: x::CURRENT_TIME};
}

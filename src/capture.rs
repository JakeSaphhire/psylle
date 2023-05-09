use rdev::{grab, Event, EventType, Key};
use std::{thread, time};


pub fn capture() -> () {
    // See https://github.com/Narsil/rdev/issues/101#issuecomment-1500698317 and 
    // and https://github.com/jersou/mouse-actions/blob/7bd717d32408d1b836e031531f1d051b51957e04/src/main.rs#L33
    thread::sleep(time::Duration::from_millis(300));
    static mut EXIT: bool = false;
    loop {
        let callback = | event: Event| -> Option<Event> {
            match event.event_type {
                EventType::KeyPress(Key::Escape) => {unsafe{EXIT = true}; Some(event)},
                _ => {/*Add the event to the event queue*/;None},
            }
        };
        let grabber = std::thread::spawn( move || -> () {
            if let Err(e) = grab(callback) {
                println!("Grabbing error: {:?}", e);
            }
            unsafe{
                if EXIT == true {()}
            }
        });
        grabber.join().expect("Panic on join");
    }
}
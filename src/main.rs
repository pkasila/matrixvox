mod samples;
mod animations;
mod net;

#[macro_use]
extern crate serde_derive;

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::{io};
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};
use rppal::gpio::{Gpio, Trigger};

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // GPIO
    let gpio = Gpio::new().unwrap();
    let mut interrupt_pin = gpio.get(5).unwrap().into_input();

    let mut slice = 0;
    let mut frame = 0;

    let mut latest_interrupt = 0;

    let mut frames = 0;
    let mut slices = 0;

    let mut f = OpenOptions::new().write(true).read(false).open("/dev/tpic6c595.0").unwrap();

    interrupt_pin.set_async_interrupt(Trigger::FallingEdge, move |_| {

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let delta = now.as_nanos() - latest_interrupt;
        latest_interrupt = now.as_nanos();

        if delta > 1000000000 || delta < 800000 {
            return;
        }

        if delta > 6800000 {
            slice = 0;
            frame += 1;
        } else if delta > 800000 {
            slice += 1;
        }

        f.write(&anim[frame % frames][slice % (slices * 2)]).unwrap();
    }).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let mut s_anim = animations::horizontal_plane::BITMAP;

        let mut anim: Vec<Vec<[u8; 8]>> = vec![];

        for mut s_data in s_anim {
            let mut data: Vec<[u8; 8]> = Vec::from(s_data);
            s_data.reverse();
            data.append(&mut Vec::from(s_data));
            let mut d = vec![data];
            anim.append(&mut d);
        }

        println!("Connection established!");
    }

    pause();
}

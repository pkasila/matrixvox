use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rppal::gpio::{Gpio, Level, Trigger};
use rppal::gpio::Trigger::RisingEdge;

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
    // cycles counter
    let cycles = Arc::new(Mutex::new(0_u64));
    let cycles_2 = Arc::clone(&cycles);

    // period
    let period = Arc::new(Mutex::new(1000000000 / 12));
    let period_2 = Arc::clone(&period);

    // GPIO
    let gpio = Gpio::new().unwrap();
    let mut interrupt_pin = gpio.get(5).unwrap().into_input();

    // Slice
    let slice = Arc::new(Mutex::new(0));
    let slice_2 = Arc::clone(&slice);

    // Sum
    let mut sum = 0;

    let latest_interrupt = Arc::new(Mutex::new(0));

    interrupt_pin.set_async_interrupt(Trigger::FallingEdge, move |_| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let delta = now.as_nanos() - *latest_interrupt.lock().unwrap();
        *latest_interrupt.lock().unwrap() = now.as_nanos();



        if delta > 1000000000 {
            return;
        }

        if delta > 42 * 1000000 {
            sum += delta;
            *cycles_2.lock().unwrap() += 1;

            *slice.lock().unwrap() = 0; // set slice to 0

            if *cycles_2.lock().unwrap() % 24 == 0 {
                let d = sum / 2 / 24;
                sum = 0;
                println!("Recalc {}", d);
                // calculate frequency
                *period_2.lock().unwrap() = d; // update frequency
            }
        }
    }).unwrap();

    thread::spawn(move || {
        loop {
            println!("Cycles: {}", *cycles.lock().unwrap());
            thread::sleep(Duration::new(1,0))
        }
    });

    let mut s_data: [[u8; 8]; 8] = [
        [
            0b00000000,
            0b01111110,
            0b01000010,
            0b01000010,
            0b01000010,
            0b01000010,
            0b01111110,
            0b00000000,
        ],
        [
            0b00000000,
            0b01000010,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b01000010,
            0b00000000,
        ],
        [
            0b00000000,
            0b01000010,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b01000010,
            0b00000000,
        ],
        [
            0b00000000,
            0b01000010,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b01000010,
            0b00000000,
        ],
        [
            0b00000000,
            0b01000010,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b01000010,
            0b00000000,
        ],
        [
            0b00000000,
            0b01000010,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b01000010,
            0b00000000,
        ],
        [
            0b00000000,
            0b01000010,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b01000010,
            0b00000000,
        ],
        [
            0b00000000,
            0b01111110,
            0b01000010,
            0b01000010,
            0b01000010,
            0b01000010,
            0b01111110,
            0b00000000,
        ],
    ];
    s_data.reverse();

    let mut rev_data = Vec::from( s_data);
    rev_data.reverse();

    let mut data = Vec::from(s_data);
    data.append(&mut rev_data);

    thread::spawn(move || {
        let mut f = OpenOptions::new().write(true).read(false).open("/dev/tpic6c595.0").unwrap();
        loop {
            let s = *slice_2.lock().unwrap();
            *slice_2.lock().unwrap() += 1;
            f.write(&data[s % 16]).unwrap();
            let p = *period.lock().unwrap();
            thread::sleep(Duration::new(0, (p / 8) as u32));
        }
    });

    pause();
}

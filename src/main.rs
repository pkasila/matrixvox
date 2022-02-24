mod samples;
mod animations;
mod net;

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::{io};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use rppal::gpio::{Gpio, Trigger};
use futures::prelude::*;
use tokio::net::TcpListener;
use tokio_serde::formats::SymmetricalMessagePack;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use crate::net::device_information::DeviceInformation;
use crate::net::pack::Pack;

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

#[tokio::main]
pub async fn main() {

    // GPIO
    let gpio = Gpio::new().unwrap();
    let mut interrupt_pin = gpio.get(5).unwrap().into_input();

    let mut slice = 0;
    let mut frame = 0;

    let mut latest_interrupt = 0;

    let frames = Arc::new(Mutex::new(0));
    let frames_2 = frames.clone();
    let slices = Arc::new(Mutex::new(0));
    let slices_2 = slices.clone();

    let init = Arc::new(Mutex::new(false));
    let init_2 = init.clone();

    let mut f = OpenOptions::new().write(true).read(false).open("/dev/tpic6c595.0").unwrap();

    let anim: Arc<Mutex<Vec<Vec<[u8; 8]>>>> = Arc::new(Mutex::new(vec![]));
    let anim_2 = anim.clone();

    interrupt_pin.set_async_interrupt(Trigger::FallingEdge, move |_| {
        if !*init.lock().unwrap() {
            return;
        }

        let frames = *frames.lock().unwrap();
        let slices = *slices.lock().unwrap();

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

        f.write(&anim.lock().unwrap()[frame % frames][slice % (slices * 2)]).unwrap();
    }).unwrap();

    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();

    println!("listening on {:?}", listener.local_addr());

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let (read, write) = socket.into_split();

        let ld = FramedWrite::new(write, LengthDelimitedCodec::new());
        let mut serialized = tokio_serde::SymmetricallyFramed::new(
            ld,
            SymmetricalMessagePack::<DeviceInformation>::default(),
        );

        serialized.send(DeviceInformation {
            product_id: "testing".to_string(),
            serial_number: "testing".to_string(),
            vox_size: [8, 8, 16],
        }).await.unwrap();

        // Delimit frames using a length header
        let length_delimited = FramedRead::new(read, LengthDelimitedCodec::new());

        // Deserialize frames
        let mut deserialized = tokio_serde::SymmetricallyFramed::new(
            length_delimited,
            SymmetricalMessagePack::<Pack>::default(),
        );

        // Spawn a task that prints all received messages to STDOUT
        while let Some(msg) = deserialized.try_next().await.unwrap() {
            let pack: Pack = msg;

            let mut anim_c: Vec<Vec<[u8; 8]>> = vec![];

            for mut s_data in pack.data {
                let mut data: Vec<[u8; 8]> = s_data.clone();
                s_data.reverse();
                data.append(&mut s_data);
                let mut d = vec![data];
                anim_c.append(&mut d);
            }

            *anim_2.lock().unwrap() = anim_c;
            *frames_2.lock().unwrap() = pack.anim_rate;
            *slices_2.lock().unwrap() = pack.slices;
            *init_2.lock().unwrap() = true;
        }
    }
}

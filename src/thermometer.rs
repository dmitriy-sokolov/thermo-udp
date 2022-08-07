use std::{
    error::Error,
    net::{ToSocketAddrs, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::temperature::Temperature;

pub struct Thermometer {
    temperature: Arc<Temperature>,
    finished: Arc<AtomicBool>,
}

impl Thermometer {
    pub fn new(address: impl ToSocketAddrs) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(address)?;
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;

        let finished = Arc::new(AtomicBool::new(false));
        let temperature = Arc::new(Temperature::default());

        let temperature_clone = temperature.clone();
        let finished_clone = finished.clone();
        thread::spawn(move || loop {
            if finished_clone.load(Ordering::SeqCst) {
                return;
            }

            let mut buf = [0; 4];
            if let Err(err) = socket.recv_from(&mut buf) {
                println!("can't receive datagram: {err}");
            }

            let val = f32::from_be_bytes(buf);
            temperature_clone.set(val);
        });

        Ok(Self {
            temperature,
            finished,
        })
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature.get()
    }
}

impl Drop for Thermometer {
    fn drop(&mut self) {
        self.finished.store(true, Ordering::SeqCst)
    }
}

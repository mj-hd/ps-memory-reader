use std::{process::Output, thread::sleep, time::Duration};

use anyhow::{bail, Result};
use log::{trace, warn};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rppal::{
    gpio::{Gpio, InputPin, OutputPin},
    spi::{reverse_bits, Bus, Spi},
};

enum Pin {
    Ack = 21,
}

pub struct Io {
    gpio: Gpio,
    spi: Spi,

    ack: InputPin,
}

impl Io {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new()?;
        let spi = Spi::new(
            Bus::Spi0,
            rppal::spi::SlaveSelect::Ss0,
            250000,
            rppal::spi::Mode::Mode3,
        )?;

        let mut ack = (&gpio).get(Pin::Ack as u8)?.into_input_pullup();
        ack.set_interrupt(rppal::gpio::Trigger::FallingEdge)?;

        Ok(Self { gpio, spi, ack })
    }

    pub fn read_sector(&mut self, sector: u16) -> Result<[u8; 128]> {
        self.request_access()?;
        self.request_read()?;
        self.receive_ids()?;
        self.request_sector(sector)?;
        self.receive_acks()?;
        let confirm_sector = self.receive_confirm_sector()?;
        let data = self.receive_data()?;
        let checksum = self.receive_checksum()?;
        self.receive_end()?;

        // TODO: checksum
        println!(
            "received sector={}, total {}bytes, checksum=0x{:02x}",
            confirm_sector,
            data.len(),
            checksum
        );

        Ok(data)
    }

    fn send(&mut self, req: u8) -> Result<u8> {
        let mut data = [(req as u8)];
        let mut buffer = [0; 1];

        reverse_bits(&mut data);

        self.ack
            .poll_interrupt(false, Some(Duration::from_millis(20)))?;

        trace!("send transfer: {:?}(0x{:02x})", data, req);

        self.spi.transfer(&mut buffer, &data)?;

        reverse_bits(&mut buffer);

        let res = buffer[0];

        trace!("send received: {:?}(0x{:02x})", buffer, res);

        self.ack
            .poll_interrupt(false, Some(Duration::from_millis(20)))?;

        Ok(res)
    }

    fn receive(&mut self) -> Result<u8> {
        self.send(0x00)
    }

    fn send_req(&mut self, req: Req) -> Result<u8> {
        self.send(req as u8)
    }

    fn receive_res(&mut self) -> Result<Res> {
        Ok(Res::from_u8(self.receive()?).unwrap_or(Res::Unknown))
    }

    fn request_access(&mut self) -> Result<()> {
        trace!("request access");

        self.send_req(Req::Access)?;

        trace!("request access finished");

        Ok(())
    }

    fn request_read(&mut self) -> Result<()> {
        trace!("request read");

        let flags = self.send_req(Req::Read)?;

        trace!("request read finished: FLAGS=0x{:02x}", flags);

        Ok(())
    }

    fn request_sector(&mut self, sector: u16) -> Result<()> {
        trace!("request sector");

        let res1 = self.send((sector >> 8) as u8)?;
        let res2 = self.send((sector & 0xFF) as u8)?;

        trace!("request sector finished: RES=0x{:02x} 0x{:02x}", res1, res2);

        Ok(())
    }

    fn receive_ids(&mut self) -> Result<()> {
        trace!("receive ids");

        let res1 = self.receive_res()?;
        let res2 = self.receive_res()?;

        trace!("receive ids finished: RES={:?} {:?}", res1, res2);

        if res1 != Res::MemCard1 {
            warn!("unexpected res {:?}", res1);
        }

        if res2 != Res::MemCard2_Ack2 {
            warn!("unexpected res {:?}", res2);
        }

        Ok(())
    }

    fn receive_acks(&mut self) -> Result<()> {
        trace!("receive acks");

        let res1 = self.receive_res()?;
        let res2 = self.receive_res()?;

        trace!("receive acks finished: RES={:?} {:?}", res1, res2);

        if res1 != Res::Ack1 {
            warn!("unexpected res {:?}", res1);
        }

        if res2 != Res::MemCard2_Ack2 {
            warn!("unexpected res {:?}", res2);
        }

        Ok(())
    }

    fn receive_confirm_sector(&mut self) -> Result<u16> {
        trace!("receive confirm sector");

        let res1 = self.receive()?;
        let res2 = self.receive()?;

        trace!(
            "receive confirm sector finished: RES=0x{:02x} 0x{:02x}",
            res1,
            res2
        );

        Ok((res1 as u16) << 8 | (res2 as u16))
    }

    fn receive_data(&mut self) -> Result<[u8; 128]> {
        let mut result = [0; 128];

        for i in 0..128 {
            let data = self.receive()?;
            trace!("receive data @{}: 0x{:02x}", i, data);

            result[i] = data;
        }

        Ok(result)
    }

    fn receive_checksum(&mut self) -> Result<u8> {
        trace!("receive checksum");

        let checksum = self.receive()?;

        trace!("receive checksum finished: RES=0x{:02x}", checksum,);

        Ok(checksum)
    }

    fn receive_end(&mut self) -> Result<()> {
        trace!("receive end");

        let res = self.receive_res()?;

        trace!("receive end finished: RES={:?}", res);

        if res != Res::Good {
            warn!("unexpected end: {:?}", res);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Req {
    Access = 0x81,
    Read = 0x52,
    Write = 0x57,
}

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq)]
enum Res {
    Unknown,
    MemCard1 = 0x5A,
    Ack1 = 0x5C,
    MemCard2_Ack2 = 0x5D,
    Good = 0x47,
    BadChecksum = 0x4E,
    BadSector = 0xFF,
}

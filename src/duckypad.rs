use std::fmt;

use chrono::{DateTime, Local};
use hidapi::{HidApi, HidDevice, HidError};
use strum_macros::FromRepr;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum DuckyPadKind {
    DuckyPad,
    DuckyPadPro,
}

impl fmt::Display for DuckyPadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuckyPad => write!(f, "duckyPad"),
            Self::DuckyPadPro => write!(f, "duckyPad Pro"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DuckyPadInfo {
    pub fw_ver_major: u8,
    pub fw_ver_minor: u8,
    pub fw_ver_patch: u8,
    pub hardware_rev: DuckyPadKind,
    pub serial_no: u32,
    pub curr_profile: u8,
    pub is_sleeping: bool,
    pub is_rtc_valid: bool,
    pub utc_offset: u16,
    pub unix_timestamp: u32,
}

impl fmt::Display for DuckyPadInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Firmware Major Version: {}", self.fw_ver_major)?;
        writeln!(f, "Firmware Minor Version: {}", self.fw_ver_minor)?;
        writeln!(f, "Firmware Patch Version: {}", self.fw_ver_patch)?;
        writeln!(f, "Hardware Revision: {}", self.hardware_rev)?;
        writeln!(f, "Serial Number: {:x}", self.serial_no)?;
        writeln!(f, "Current Profile: {}", self.curr_profile)?;
        writeln!(f, "Is Sleeping: {}", self.is_sleeping)?;
        if self.is_rtc_valid {
            writeln!(f, "Valid RTC: {}", self.is_rtc_valid)?;
            writeln!(f, "UTC Offset: {}", self.utc_offset)?;
            write!(f, "Unix Timestamp: {}", self.unix_timestamp)?;
        } else {
            write!(f, "Valid RTC: {}", self.is_rtc_valid)?;
        }

        Ok(())
    }
}

impl From<[u8; 64]> for DuckyPadInfo {
    fn from(from: [u8; 64]) -> Self {
        Self {
            fw_ver_major: from[3],
            fw_ver_minor: from[4],
            fw_ver_patch: from[5],
            hardware_rev: match from[6] {
                20 => DuckyPadKind::DuckyPad,
                24 => DuckyPadKind::DuckyPadPro,
                _ => panic!("Invalid hardware revision")
            },
            serial_no: {
                let mut bytes: [u8; 4] = [0, 0, 0, 0];
                bytes.copy_from_slice(&from[7..11]);
                u32::from_le_bytes(bytes)
            },
            curr_profile: from[11],
            is_sleeping: from[12] != 0,
            is_rtc_valid: from[13] != 0,
            utc_offset: {
                let mut bytes: [u8; 2] = [0, 0];
                bytes.copy_from_slice(&from[14..16]);
                u16::from_le_bytes(bytes)
            },
            unix_timestamp: {
                let mut bytes: [u8; 4] = [0, 0, 0, 0];
                bytes.copy_from_slice(&from[16..20]);
                u32::from_le_bytes(bytes)
            }, 
        }
    }
}

#[derive(FromRepr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LedIdx {
    Led0 = 0,
    Led1 = 1,
    Led2 = 2,
    Led3 = 3,
    Led4 = 4,
    Led5 = 5,
    Led6 = 6,
    Led7 = 7,
    Led8 = 8,
    Led9 = 9,
    Led10 = 10,
    Led11 = 11,
    Led12 = 12,
    Led13 = 13,
    Led14 = 14,
    Led15 = 15,
    Led16 = 16,
    Led17 = 17,
    Led18 = 18,
    Led19 = 19,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn from_str<A: AsRef<str>>(s: A) -> Option<Color> {
        let st = s.as_ref();

        if !st.is_ascii() || st.len() != 6 {
            return None
        }

        let red = u8::from_str_radix(&st[0..2], 16).ok()?;
        let green = u8::from_str_radix(&st[2..4], 16).ok()?;
        let blue = u8::from_str_radix(&st[4..6], 16).ok()?;

        Some(Color { red, green, blue })
    }
}

/// The index of a persistent global variable.
#[derive(FromRepr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum PgvIdx {
    Pgv0 = 0,
    Pgv1 = 1,
    Pgv2 = 2,
    Pgv3 = 3,
    Pgv4 = 4,
    Pgv5 = 5,
    Pgv6 = 6,
    Pgv7 = 7,
    Pgv8 = 8,
    Pgv9 = 9,
    Pgv10 = 10,
    Pgv11 = 11,
    Pgv12 = 12,
    Pgv13 = 13,
    Pgv14 = 14,
    Pgv15 = 15,
    Pgv16 = 16,
    Pgv17 = 17,
    Pgv18 = 18,
    Pgv19 = 19,
    Pgv20 = 20,
    Pgv21 = 21,
    Pgv22 = 22,
    Pgv23 = 23,
    Pgv24 = 24,
    Pgv25 = 25,
    Pgv26 = 26,
    Pgv27 = 27,
    Pgv28 = 28,
    Pgv29 = 29,
    Pgv30 = 30,
    Pgv31 = 31,
}

impl PgvIdx {
    pub const MAX: usize = 31;
}

/// A persistent global variable.
#[derive(Debug, Clone, Copy)]
pub struct Pgv {
    pub idx: PgvIdx,
    pub value: u32,
}

impl fmt::Display for Pgv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} = {}", self.idx as u8, self.value)
    }
}

/// An iterator over dumped PGVs.
pub struct PgvDump {
    pgvidx: usize,
    startidx: usize,
    raw: [u8; 64]
}

impl PgvDump {
    pub(self) fn new(pgvidx: usize, raw: [u8; 64]) -> Self {
        Self { pgvidx, startidx: 0, raw }
    }
}

impl Iterator for PgvDump {
    type Item = Pgv;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pgvidx > PgvIdx::MAX { return None }

        const OFFSET: usize = 3;
        const LAST_START: usize = 59;
        let startidx = (4 * self.startidx) + OFFSET;

        if startidx > LAST_START { return None }

        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.raw[startidx..startidx+4]);
        let value = u32::from_le_bytes(bytes);

        let idx = PgvIdx::from_repr(self.pgvidx as u8)?;

        self.pgvidx += 1;
        self.startidx += 1;

        Some(Pgv {idx, value})
    }
}

#[derive(Debug)]
pub struct DuckyPad {
    device: HidDevice,
}

impl DuckyPad {
    const VENDOR_ID: u16 = 0x0483;
    const DEVICE_ID: u16 = 0xd11d;
    const USAGE_ID: u8 = 0x05;
    const MESSAGE_TMPL: [u8; 64] = {
        let mut a = [0u8; 64];
        a[0] = Self::USAGE_ID;
        a
    };

    pub fn new() -> Result<Self, DuckyPadError> {
        let device = HidApi::new()?.open(Self::VENDOR_ID, Self::DEVICE_ID)?;

        Ok(Self { device } )
    }

    pub fn query_info(&self) -> Result<DuckyPadInfo, DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x00;
        
        let resp = self.run_command(&msg, true)?;

        Ok(DuckyPadInfo::from(resp))
    }

    pub fn goto_profile_num(&self, id: u8) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x01;
        msg[3] = id;
        
        self.run_command(&msg, true)?;

        Ok(())
    }

    pub fn goto_profile_name(&self, name: &str) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x17;

        if name.len() >= 60 {
            return Err(DuckyPadError::NameTooLong(name.to_string()));
        }

        if !name.is_ascii() {
            return Err(DuckyPadError::NotASCII(name.to_string()));
        }

        let bytes = name.as_bytes();
        let dst = &mut msg[3..];
        dst[..bytes.len()].copy_from_slice(bytes);

        self.run_command(&msg, true)?;

        Ok(())
    }

    pub fn prev_profile(&self) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x02;
        
        self.run_command(&msg, true)?;

        Ok(())
    }

    pub fn next_profile(&self) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x03;
        
        self.run_command(&msg, true)?;

        Ok(())
    }

    pub fn set_rgb(&self, idx: LedIdx, col: Color) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x04;

        msg[3] = idx as u8;
        msg[4] = col.red;
        msg[5] = col.green;
        msg[6] = col.blue;
        
        self.run_command(&msg, true)?;

        Ok(())
    }

    pub fn reset(&self, msc: bool) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x14;
        if msc {
            msg[3] = 1;
        }
        
        self.run_command(&msg, false)?;

        Ok(())
    }

    pub fn sleep(&self) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x15;
        
        self.run_command(&msg, true)?;

        Ok(())
    }

    pub fn wake(&self) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x16;
        
        self.run_command(&msg, true)?;

        Ok(())
    }

    pub fn dump_pgv(&self, idx: PgvIdx) -> Result<PgvDump, DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x18;
        msg[3] = idx as u8;

        let resp = self.run_command(&msg, true)?;

        Ok(PgvDump::new(idx as usize, resp))
    }

    pub fn write_pgv(&self, pgvs: &[Pgv]) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x19;

        const OFFSET: usize = 3;
        for (idx, pgv) in pgvs.iter().enumerate() {
            // assert here as it should have been caught by the frontend
            assert!(idx < 10, "cannot write more than 10 pgvs at once");

            // get our current offset linearly (5i + 3)
            let curr_offset = (5 * (idx)) + OFFSET;

            // set the pgv index
            msg[curr_offset] = (pgv.idx as u8) + 128;

            // set the pgv value
            let pgv_raw =  pgv.value.to_le_bytes();
            msg[curr_offset+1..curr_offset+5].copy_from_slice(&pgv_raw);
        }

        self.run_command(&msg, true)?;
        
        Ok(())
    }

    pub fn update_rtc(&self, time: Option<DateTime<Local>>) -> Result<(), DuckyPadError> {
        let mut msg = Self::MESSAGE_TMPL;
        msg[2] = 0x1a;

        let now = if let Some(time) = time { time } else { Local::now() };

        let timestamp = now.timestamp() as i32;
        let offset_mins = (now.offset().local_minus_utc() / 60) as i16;

        let timestamp_raw = timestamp.to_le_bytes();
        let offset_raw = offset_mins.to_le_bytes();

        msg[3..7].copy_from_slice(&timestamp_raw);
        msg[7..9].copy_from_slice(&offset_raw);
        
        self.run_command(&msg, true)?;

        Ok(())
    }

    fn run_command(&self, msg: &[u8], wait: bool) -> Result<[u8; 64], DuckyPadError> {
        eprintln!("sending command {:?}", msg);
        self.device.write(msg)?;
        // if we don't wait for the response, return immediately
        // this is exclusively for the reset command, since there's nothing to read
        if !wait {
            return Ok([0; 64]);
        }

        let mut resp = [0u8; 64];

        self.device.read(&mut resp)?;

        eprintln!("got response {:?}", resp);

        if resp[2] == 1 {
            return Err(DuckyPadError::PadError);
        } else if resp[2] == 2 {
            return Err(DuckyPadError::PadBusy);
        } else if resp[2] > 0 {
            panic!("unknown duckyPad status response");
        }

        Ok(resp)
    } 
}

#[derive(Error, Debug)]
pub enum DuckyPadError {
    #[error("{0}")]
    HidError(#[from] HidError),
    #[error("DuckyPad is busy")]
    PadError,
    #[error("DuckyPad returned an error")]
    PadBusy,
    #[error("Invalid profile name: {0} is longer than 59 characters")]
    NameTooLong(String),
    #[error("Invalid profile name: {0} is not valid ASCII")]
    NotASCII(String),
}
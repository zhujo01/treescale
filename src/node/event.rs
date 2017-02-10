#![allow(dead_code)]
use std::io::{Result, Error, ErrorKind};
use helpers::{parse_number, encode_number, parse_number64, encode_number64, Path};

pub struct Event {
    pub path: Path,
    pub name: String,
    pub from: u64,
    pub target: String,
    pub public_data: String,
    pub data: Vec<u8>,
}

impl Event {
    #[inline(always)]
    pub fn default() -> Event {
        Event {
            path: Path::new(),
            name: String::new(),
            from: 0,
            target: String::new(),
            public_data: String::new(),
            data: vec![],
        }
    }

    #[inline(always)]
    pub fn from_raw(data: &Vec<u8>) -> Result<Event> {
        let mut offset = 0 as usize;
        let mut ev = Event::default();
        let mut endian_bytes = vec![0; 4];
        let data_len = data.len();

        if data.len() <= 6 * 4 {
            println!("{:?}", data.len());
            return Err(Error::new(ErrorKind::InvalidData, "Event data is too short to convert it!!"));
        }

        ev.path = match Event::read_field(&data, &mut endian_bytes, data_len, offset, true) {
            Ok((_, path_buffer, off)) => {
                offset = off;
                match Path::from_bytes(path_buffer.as_slice()) {
                    Some(p) => p,
                    None => Path::new()
                }
            }
            Err(e) => return Err(e)
        };

        ev.name = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.from = match Event::read_field(&data, &mut endian_bytes, data_len, offset, true) {
            Ok((_, from_buffer, off)) => {
                offset = off;
                parse_number64(from_buffer.as_slice())
            }
            Err(e) => return Err(e)
        };

        ev.target = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.public_data = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.data = match Event::read_field(&data, &mut endian_bytes, data_len, offset, true) {
            Ok((_, f, _)) => {
                f
            }
            Err(e) => return Err(e)
        };

        Ok(ev)
    }

    pub fn to_raw(&self) -> Result<Vec<u8>> {
        let (path_len, name_len, from_len, target_len, public_data_len, event_data_len)
                    = (self.path.len(), self.name.len(), 8 /* from length */, self.target.len(), self.public_data.len(), self.data.len());

        // calculating total data length
        let data_len = 4
            + path_len + 4
            + name_len + 4
            + from_len + 4
            + target_len + 4
            + public_data_len + 4
            + event_data_len + 4;

        let mut buf: Vec<u8> = vec![0; data_len];
        let mut len_buf: Vec<u8> = vec![0; 4];
        let mut offset = 0;

        // writing full data length only
        encode_number(&mut len_buf, (data_len - 4) as u32);
        buf[0..4].copy_from_slice(len_buf.as_slice());
        offset += 4;

        // setting path data here
        let path_bytes = match self.path.to_bytes() {
            Some(b) => b,
            None => return Err(Error::new(ErrorKind::InvalidData, "Unable to convert path to bytes"))
        };

        match Event::write_field(&mut len_buf, &mut buf, path_bytes.as_slice(), path_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + path_len;

        // setting name data here
        match Event::write_field(&mut len_buf, &mut buf, self.name.as_bytes(), name_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + name_len;

        // setting "from" data here
        let mut from_buf = vec![0u8; 8];
        encode_number64(&mut from_buf, self.from);
        match Event::write_field(&mut len_buf, &mut buf, from_buf.as_slice(), from_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + from_len;

        // setting target data here
        match Event::write_field(&mut len_buf, &mut buf, self.target.as_bytes(), target_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + target_len;

        // setting public_data data here
        match Event::write_field(&mut len_buf, &mut buf, self.public_data.as_bytes(), public_data_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + public_data_len;

        // setting "data" data here
        match Event::write_field(&mut len_buf, &mut buf, self.data.as_slice(), event_data_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        // offset += 4 + data_len;

        Ok(buf)
    }

    #[inline(always)]
    fn read_field(data: &Vec<u8>, endian_bytes: &mut Vec<u8>, data_len: usize, off: usize, get_vec: bool) -> Result<(String, Vec<u8>, usize)> {
        let mut offset = off as usize;
        for i in 0..4 {
            endian_bytes[i] = data[offset + i]
        }

        offset += 4;
        let endian_len = parse_number(endian_bytes.as_slice()) as usize;
        if endian_len > (data_len - offset) {
            return Err(Error::new(ErrorKind::InvalidData, "error decoding given data"));
        }

        let d = Vec::from(&data[offset..offset + endian_len]);
        if get_vec {
            return Ok((String::new(), d, offset + endian_len));
        }

        Ok(match String::from_utf8(d) {
            Ok(s) => (s, vec![], offset + endian_len),
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Unable to convert data to string"))
        })
    }

    #[inline(always)]
    fn write_field(len_buf: &mut Vec<u8>, buf: &mut Vec<u8>, data: &[u8], filed_len: usize, offset: usize) -> Result<()> {
        // Writing Path
        encode_number(len_buf, (filed_len) as u32);
        let mut off = offset;
        buf[off..off + 4].copy_from_slice(len_buf.as_slice());
        off += 4;
        buf[off..off + filed_len].copy_from_slice(data);
        Ok(())
    }
}

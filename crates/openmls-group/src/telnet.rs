use std::io;
use tokio_util::{bytes::Buf, codec::Decoder};

pub struct TelnetCodec {
    current_line: Vec<u8>,
}

impl TelnetCodec {
    pub fn new() -> Self {
        TelnetCodec {
            current_line: Vec::with_capacity(1024),
        }
    }
}

#[derive(Debug)]
pub enum Item {
    Line(Vec<u8>),
    PublishKeyPackage,
    CreateGroup(Vec<u8>),
    ShowKPDetails,
    SE,
    DataMark,
    Break,
    InterruptProcess,
    AbortOutput,
    AreYouThere,
    GoAhead,
    SB,
    Will(u8),
    Wont(u8),
    Do(u8),
    Dont(u8),
}

impl Decoder for TelnetCodec {
    type Item = Item;
    type Error = io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if src.is_empty() {
                return Ok(None);
            }

            if src[0] == 0xff {
                let (res, consume) = try_parse_iac(src.chunk());
                src.advance(consume);

                match res {
                    ParseIacResult::Invalid(err) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, err));
                    }
                    ParseIacResult::NeedMore => return Ok(None),
                    ParseIacResult::Item(item) => return Ok(Some(item)),
                    ParseIacResult::NOP => { /* go around loop */ }
                    ParseIacResult::EraseCharacter => {
                        self.current_line.pop();
                    }
                    ParseIacResult::EraseLine => {
                        self.current_line.clear();
                    }
                    ParseIacResult::Escaped => {
                        self.current_line.push(0xff);
                    }
                }
            } else {
                let byte = src.get_u8();

                match byte {
                    10 => {
                        let line = self.current_line.to_vec();
                        self.current_line.clear();
                        let item = parse_line(line);

                        return Ok(item);
                    }
                    0..=31 => {
                        // ignore
                    }
                    _ => self.current_line.push(byte),
                }
            }
        }
    }
}

enum ParseIacResult {
    Invalid(String),
    NeedMore,
    Item(Item),
    NOP,
    EraseCharacter,
    EraseLine,
    Escaped,
}

fn try_parse_iac(bytes: &[u8]) -> (ParseIacResult, usize) {
    if bytes.len() < 2 {
        return (ParseIacResult::NeedMore, 0);
    }
    if bytes[0] != 0xff {
        unreachable!();
    }
    if is_three_byte_iac(bytes[1]) && bytes.len() < 3 {
        return (ParseIacResult::NeedMore, 0);
    }

    match bytes[1] {
        240 => (ParseIacResult::Item(Item::SE), 2),
        241 => (ParseIacResult::NOP, 2),
        242 => (ParseIacResult::Item(Item::DataMark), 2),
        243 => (ParseIacResult::Item(Item::Break), 2),
        244 => (ParseIacResult::Item(Item::InterruptProcess), 2),
        245 => (ParseIacResult::Item(Item::AbortOutput), 2),
        246 => (ParseIacResult::Item(Item::AreYouThere), 2),
        247 => (ParseIacResult::EraseCharacter, 2),
        248 => (ParseIacResult::EraseLine, 2),
        249 => (ParseIacResult::Item(Item::GoAhead), 2),
        250 => (ParseIacResult::Item(Item::SB), 2),
        251 => (ParseIacResult::Item(Item::Will(bytes[2])), 3),
        252 => (ParseIacResult::Item(Item::Wont(bytes[2])), 3),
        253 => (ParseIacResult::Item(Item::Do(bytes[2])), 3),
        254 => (ParseIacResult::Item(Item::Dont(bytes[2])), 3),
        255 => (ParseIacResult::Escaped, 2),
        cmd => (
            ParseIacResult::Invalid(format!("Unknown IAC command {}.", cmd)),
            0,
        ),
    }
}

fn is_three_byte_iac(byte: u8) -> bool {
    match byte {
        251..=254 => true,
        _ => false,
    }
}

// Mark: Openmls
fn parse_line(line: Vec<u8>) -> Option<Item> {
    println!("[Client] sent command in byte {:?}", line);
    // c#pkp == command: publish key package
    if line.to_vec() == [99, 34, 112, 107, 112] {
        println!("[Client] Publishing its keypackage");

        return Some(Item::PublishKeyPackage);
    }

    // c#cgw == command: [c]reate [g]roup [w]ith ids of participants
    // separates by space
    if line.to_vec()[0..4] == [99, 34, 99, 103] {
        let curr = &line[5..];
        let mut ids: Vec<u8> = Vec::new();
        for value in curr {
            // 32 is space
            if value == &32 {
                continue;
            }

            ids.push(value.clone());
        }

        return Some(Item::CreateGroup(ids));
    }

    // c#skd == command: [s]how [k]akacge [d]etails
    if line.to_vec() == [99, 34, 115, 107, 100] {
        println!("[Client] Show keypackage details");

        return Some(Item::ShowKPDetails);
    }

    return Some(Item::Line(line));
}

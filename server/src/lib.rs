#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate failure;

use std::{fmt, io, ops, result};

use crate::monitor::Monitor;
use crate::reader::{in_libc::LibcReader, Read};
use crate::utils::NumBytes;
use crate::writer::{out_simple::SimpleWriter, Write};

pub mod monitor;
pub mod reader;
pub mod utils;
pub mod writer;

use rocket::tokio::sync::broadcast::Sender;
use rocket::serde::{Serialize, Deserialize};

pub type Errno = nix::errno::Errno;
pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    IoError(io::Error),
    #[fail(display = "{}", _0)]
    NixError(nix::Error),
    #[fail(display = "{}", _0)]
    LinkStatsError(String),
    #[fail(display = "{}", _0)]
    Other(&'static str),
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Error {
        Error::NixError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct InterfaceInfoItem {
    name: String,
}

impl fmt::Display for InterfaceInfoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:<width$}",
            self.name,
            width = InterfaceStat::DISPLAY_WIDTH
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct InterfaceInfo(Vec<InterfaceInfoItem>);

impl fmt::Display for InterfaceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.0.len();
        for (i, item) in self.0.iter().enumerate() {
            write!(
                f,
                "{}{}",
                item,
                if i == len - 1 {
                    ""
                } else {
                    InterfaceStats::DELIMITER
                }
            )?
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct InterfaceStat {
    rx: NumBytes<u64>,
    tx: NumBytes<u64>,
}

impl InterfaceStat {
    const DISPLAY_WIDTH: usize =
        NumBytes::<u64>::DISPLAY_WIDTH + 1 + NumBytes::<u64>::DISPLAY_WIDTH;
}

impl fmt::Display for InterfaceStat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.rx, self.tx)
    }
}

impl ops::Sub for &InterfaceStat {
    type Output = InterfaceStat;
    fn sub(self, other: &InterfaceStat) -> Self::Output {
        InterfaceStat {
            rx: self.rx - other.rx,
            tx: self.tx - other.tx,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct InterfaceStats(Vec<Option<InterfaceStat>>);

impl InterfaceStats {
    const DELIMITER: &'static str = " | ";

    fn empty(len: usize) -> InterfaceStats {
        InterfaceStats(vec![None; len])
    }
}

impl ops::Sub for &InterfaceStats {
    type Output = InterfaceStats;
    fn sub(self, other: &InterfaceStats) -> Self::Output {
        assert_eq!(self.0.len(), other.0.len());
        InterfaceStats(
            self.0
                .iter()
                .enumerate()
                .map(|(i, stat)| {
                    if let Some(stat) = stat {
                        if let Some(other_stat) = &other.0[i] {
                            return Some(stat - other_stat);
                        }
                    }
                    None
                })
                .collect(),
        )
    }
}

impl fmt::Display for InterfaceStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.0.len();
        for (i, stat) in self.0.iter().enumerate() {
            match stat {
                Some(stat) => write!(f, "{}", stat)?,
                None => write!(
                    f,
                    "{:<w$} {:<w$}",
                    "None",
                    "None",
                    w = NumBytes::<u64>::DISPLAY_WIDTH
                )?,
            }
            write!(f, "{}", if i == len - 1 { "" } else { Self::DELIMITER })?
        }
        Ok(())
    }
}

pub fn run(writer: Sender<Message>) -> Result<()> {
    let reader: Box<dyn Read + Send> = Box::new(LibcReader::new()?);

    let writer: Box<dyn Write> = Box::new(SimpleWriter::new(
            writer,
            reader.get_info(),
            reader.read(),
        )?);

    let mut monitor = Monitor::new(reader, writer);
    monitor.run()
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    pub info: InterfaceInfo,
    pub stats: InterfaceStats,
}

//! Input bandwidth from libc getifaddr function.

use libc::c_void;
use nix::net::if_::InterfaceFlags;
use nix::sys::socket::{AddressFamily, SockAddr};
use std::{ffi, ptr};

use crate::reader::Read;
use crate::utils::NumBytes;
use crate::Result;
use crate::{InterfaceInfo, InterfaceInfoItem, InterfaceStat, InterfaceStats};

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct IfData {
    ifi_ibytes: u32,
    ifi_obytes: u32,
}

impl IfData {
    #[cfg(target_os = "linux")]
    unsafe fn from_ifa_data(ifa_data: *mut c_void) -> Option<IfData> {
        use super::link::{LinkStats, LINK_STATS32_LEN};

        if ifa_data.is_null() {
            return None;
        }

        let data_bytes: &[u8; LINK_STATS32_LEN] = &*(ifa_data as *const [u8; LINK_STATS32_LEN]);
        match LinkStats::from_bytes(data_bytes) {
            Err(_) => None,
            Ok(link_stats) => Some(IfData {
                ifi_ibytes: link_stats.rx_bytes,
                ifi_obytes: link_stats.tx_bytes,
            }),
        }
    }

    #[cfg(not(target_os = "linux"))]
    unsafe fn from_ifa_data(ifa_data: *mut c_void) -> Option<IfData> {
        use libc::if_data;

        if ifa_data.is_null() {
            return None;
        }

        let data: if_data = *(ifa_data as *const if_data);
        Some(IfData {
            ifi_ibytes: data.ifi_ibytes,
            ifi_obytes: data.ifi_obytes,
        })
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct InterfaceAddress {
    /// Name of the network interface
    pub interface_name: String,
    /// Flags as from `SIOCGIFFLAGS` ioctl
    pub flags: InterfaceFlags,
    /// Network address of this interface
    pub address: Option<SockAddr>,
    /// Netmask of this interface
    pub netmask: Option<SockAddr>,
    /// Broadcast address of this interface, if applicable
    pub broadcast: Option<SockAddr>,
    /// Point-to-point destination address
    pub destination: Option<SockAddr>,
    /// address-family-specific data
    pub data: Option<IfData>,
}

cfg_if! {
    if #[cfg(any(target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))] {
        fn get_ifu_from_sockaddr(info: &libc::ifaddrs) -> *const libc::sockaddr {
            info.ifa_ifu
        }
    } else {
        fn get_ifu_from_sockaddr(info: &libc::ifaddrs) -> *const libc::sockaddr {
            info.ifa_dstaddr
        }
    }
}

impl InterfaceAddress {
    /// Create an `InterfaceAddress` from the libc struct.
    fn from_libc_ifaddrs(info: &libc::ifaddrs) -> InterfaceAddress {
        let ifname = unsafe { ffi::CStr::from_ptr(info.ifa_name) };
        let address = unsafe { SockAddr::from_libc_sockaddr(info.ifa_addr) };
        let netmask = unsafe { SockAddr::from_libc_sockaddr(info.ifa_netmask) };
        let data = unsafe { IfData::from_ifa_data(info.ifa_data) };

        let mut addr = InterfaceAddress {
            interface_name: ifname.to_string_lossy().to_string(),
            flags: InterfaceFlags::from_bits_truncate(info.ifa_flags as i32),
            address,
            netmask,
            broadcast: None,
            destination: None,
            data,
        };

        let ifu = get_ifu_from_sockaddr(info);
        if addr.flags.contains(InterfaceFlags::IFF_POINTOPOINT) {
            addr.destination = unsafe { SockAddr::from_libc_sockaddr(ifu) };
        } else if addr.flags.contains(InterfaceFlags::IFF_BROADCAST) {
            addr.broadcast = unsafe { SockAddr::from_libc_sockaddr(ifu) };
        }

        addr
    }
}

/// Holds the results of `getifaddrs`.
///
/// Use the function `getifaddrs` to create this Iterator. Note that the
/// actual list of interfaces can be iterated once and will be freed as
/// soon as the Iterator goes out of scope.
#[derive(Debug, Eq, Hash, PartialEq)]
struct InterfaceAddressIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

fn get_interfaces() -> Result<InterfaceAddressIterator> {
    let mut addrs: *mut libc::ifaddrs = ptr::null_mut();
    match nix::errno::Errno::result(unsafe { libc::getifaddrs(&mut addrs) }) {
        Ok(_) => Ok(InterfaceAddressIterator {
            base: addrs,
            next: addrs,
        }),
        Err(err) => Err(err.into()),
    }
}

impl Drop for InterfaceAddressIterator {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.base) };
    }
}

impl InterfaceAddressIterator {
    #[cfg(target_os = "linux")]
    const TARGET_ADDRESS_FAMILY: AddressFamily = AddressFamily::Packet;

    #[cfg(not(target_os = "linux"))]
    const TARGET_ADDRESS_FAMILY: AddressFamily = AddressFamily::Link;

    /// Returns false if `addr` should be ignored.
    fn is_target(addr: &InterfaceAddress) -> bool {
        match addr.address {
            Some(address) => {
                address.family() == Self::TARGET_ADDRESS_FAMILY
                    && addr.flags.contains(InterfaceFlags::IFF_UP)
            }
            None => false,
        }
    }
}

impl Iterator for InterfaceAddressIterator {
    type Item = InterfaceAddress;

    /// Iterates over network interfaces obtained from `getifaddrs` but filters out interfaces that
    /// is not `AddressFamily::Link` or `InterfaceFlags::IFF_UP` .
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        while let Some(ifaddr) = unsafe { self.next.as_ref() } {
            self.next = ifaddr.ifa_next;
            let addr = InterfaceAddress::from_libc_ifaddrs(ifaddr);
            if Self::is_target(&addr) {
                return Some(addr);
            }
        }
        None
    }
}

pub struct LibcReader {
    info: InterfaceInfo,
}

impl LibcReader {
    pub fn new() -> Result<LibcReader> {
        let mut info = vec![];

        for addr in get_interfaces()? {
            info.push(InterfaceInfoItem {
                name: addr.interface_name,
            });
        }

        Ok(LibcReader {
            info: InterfaceInfo(info),
        })
    }
}

impl Read for LibcReader {
    fn get_info(&self) -> &InterfaceInfo {
        &self.info
    }

    fn read(&self) -> InterfaceStats {
        let mut stats = InterfaceStats::empty(self.get_info().0.len());

        let addrs = match get_interfaces() {
            Err(_) => return stats,
            Ok(addrs) => addrs,
        };

        for addr in addrs {
            match addr.data {
                None => continue,
                Some(data) => match self.index(&addr.interface_name) {
                    None => continue,
                    Some(i) => {
                        stats.0[i] = Some(InterfaceStat {
                            rx: NumBytes::from(data.ifi_ibytes as u64),
                            tx: NumBytes::from(data.ifi_obytes as u64),
                        })
                    }
                },
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libc_reader_new() {
        let libc_reader = LibcReader::new();
        match libc_reader {
            Err(err) => assert!(false, "`LibcReader::new()` returned an error: {}", err),
            Ok(reader) => {
                let stats = reader.read();
                println!("{:?}", reader.info);
                println!("{:?}", stats);
                assert!(true)
            }
        }
    }
}

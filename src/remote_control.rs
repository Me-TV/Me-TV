/*
 *  Me TV — It's TV for me computer.
 *
 *  A GTK+/GStreamer client for watching and recording DVB.
 *
 *  Copyright © 2019  Russel Winder
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::rc::Rc;

use futures::channel::mpsc::Sender;
use glob::glob;
use libc;
use regex::Regex;

use control_window::Message;
use frontend_manager::FrontendId;
use input_event_codes;

#[derive(Debug)]
pub struct RemoteControl {
    pub frontend_ids: Vec<FrontendId>,
    pub sys_rc_path: PathBuf,
    pub device_event_path: PathBuf,
    pub device_file: File,
}

/// Create an /dev/inputs/eventsX `PathBuf` from the /sys/class/rc/rcY `PathBuf`.
///
/// This has been constructed from the data observed on Debian Sid.
/// It is assumed that all Linux post 4.6 will be the same.
fn create_event_path_from_sys_path(path: &PathBuf) -> PathBuf {
    let components = path.components().map(|x| x.as_os_str().to_str().unwrap()).collect::<Vec<&str>>();
    let interesting_bits = vec![components[3], components[4], components[5], components[7]];
    let mut event_path_string = String::from("/dev/input/by-path/pci-");
    event_path_string += interesting_bits[1];
    event_path_string += "-usb-0:";
    event_path_string += interesting_bits[3].split("-").collect::<Vec<&str>>()[1];
    event_path_string += "-event";
    PathBuf::from(event_path_string)
}

/// Parse the dvb `PathBuf` entries in a `Vec` to return a `Vec` of `FrontendId`
fn extract_frontend_from_paths(paths: &Vec<PathBuf>) -> Vec<FrontendId> {
    let re = Regex::new(r"dvb([0-9]+)\.frontend([0-9]+)").unwrap();
    let rv = paths.iter().map(|f| {
        let caps = re.captures(f.file_name().unwrap().to_str().unwrap()).unwrap();
        let adapter = caps.get(1).unwrap().as_str().parse::<u8>().unwrap();
        let frontend = caps.get(2).unwrap().as_str().parse::<u8>().unwrap();
        FrontendId{adapter, frontend}
    }).collect();
    rv
}

/// Return all the frontends associated with this remote controller.
fn find_frontends_for_remote_control(sys_rc_path: &PathBuf) -> Vec<FrontendId> {
    let mut path = sys_rc_path.to_path_buf();
    path.push("device");
    path.push("dvb");
    path.push("dvb*.frontend*");  // NB the glob symbols here are intentional!
    let frontend_paths = match glob(path.to_str().unwrap()) {
        Ok(paths) => paths.map(|x| x.unwrap()).collect::<Vec<PathBuf>>(),
        Err(e) => panic!("Glob failure: {}", e),
    };
    extract_frontend_from_paths(&frontend_paths)
}

impl RemoteControl {
    pub fn new(sys_rc_path: &PathBuf) -> RemoteControl {
        let device_event_path= match sys_rc_path.read_link() {
            Ok(path) => create_event_path_from_sys_path(&path),
            Err(e) => panic!("Could not read symbolic link for remote control: {}", e),
        };
        let device_file = OpenOptions::new()
            .read(true)
            .open(&device_event_path)
            .expect(&format!("Cannot open the event stream {}", device_event_path.to_str().unwrap()));
        let frontend_ids = find_frontends_for_remote_control(&sys_rc_path);
        RemoteControl {
            frontend_ids,
            sys_rc_path: sys_rc_path.to_path_buf(),
            device_event_path,
            device_file,
        }
    }
}

pub fn get_list_of_remote_controllers() -> Option<Vec<Rc<RemoteControl>>> {
    let rc_devices = match glob::glob("/sys/class/rc/rc*") {
        Ok(paths) => paths.map(|x| x.unwrap()).collect::<Vec<PathBuf>>(),
        Err(e) => panic!("Glob failure: {}", e),
    };
    if  rc_devices.is_empty() { None }
    else { Some(rc_devices.iter()
        .filter(|d| find_frontends_for_remote_control(d).len() > 0)
        .map(|d| Rc::new(RemoteControl::new(d)))
        .collect::<Vec<Rc<RemoteControl>>>()) }
}

/// A keystroke intended for a given frontend for use in sending messages between the
/// remote controller daemon and the GUI.
#[derive(Debug)]
struct TargettedKeystroke {
    frontend_id: FrontendId,
    keystroke: u32,
}

/// Print all the events currently available on the event special file.
fn process_events_for_device(device: &File, frontend_ids: &Vec<FrontendId>, to_cw: &Sender<Message>) {
    // TODO is it reasonable to assume less than 64 events?
    let buffer = [libc::input_event{time: libc::timeval{tv_sec: 0, tv_usec: 0}, type_: 0, code: 0, value: 0}; 64];
    let item_size = std::mem::size_of::<libc::input_event>();
    let rc = unsafe {
        libc::read(device.as_raw_fd(), buffer.as_ptr() as *mut libc::c_void, item_size * 64)
    };
    if rc < 0 { panic!("Read failed:"); }
    let event_count = rc as usize /  item_size;
    assert_eq!(item_size * event_count, rc as usize);
    for i in 0 .. event_count {
        let item = buffer[i];
        if item.type_ == input_event_codes::EV_KEY as u16 {
            println!("Got a keystroke code {:?} and value {:?} for frontends {:?}", item.code, item.value, frontend_ids);
            // TODO Send the TargettedKeystroke to the GUI thread.
        }
    }
}

pub fn run(to_cw: Sender<Message>) {
    ioctl_write_int!(ioctl_eviocgrab, b'E', 0x90);
    loop {
        // TODO What happens if a new adapter is inserted before a remote control event happens.
        //    Need to take this into accounts, which means a listener for the remote controllers.
        let remote_controls = get_list_of_remote_controllers().unwrap_or(vec![]);
        let event_devices = remote_controls.iter().map(|d| &d.device_file).collect::<Vec<&File>>();
        let mut pollfds = event_devices.iter().map(|device| {
            unsafe {
                ioctl_eviocgrab(device.as_raw_fd(), 1).unwrap();
            }
            libc::pollfd{fd: device.as_raw_fd(), events: libc::POLLIN, revents: 0}
        }).collect::<Vec<libc::pollfd>>();
        assert_eq!(event_devices.len(), pollfds.len());
        unsafe {
            let count = libc::poll(pollfds.as_mut_ptr(), pollfds.len() as u64, -1);
            assert!(count > 0);
            for i in 0..pollfds.len() {
                if pollfds[i].revents != 0 {
                    process_events_for_device(&event_devices[i], &remote_controls[i].frontend_ids, &to_cw);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rc0_on_debian_linux() {
        assert_eq!(
            create_event_path_from_sys_path(&PathBuf::from("../../devices/pci0000:00/0000:00:14.0/usb2/2-1/2-1:1.0/rc/rc0")),
            PathBuf::from("/dev/input/by-path/pci-0000:00:14.0-usb-0:1:1.0-event"));
    }

    #[test]
    fn rc1_on_debian_linux() {
        assert_eq!(
            create_event_path_from_sys_path(&PathBuf::from("../../devices/pci0000:00/0000:00:14.0/usb2/2-3/2-3:1.0/rc/rc1")),
            PathBuf::from("/dev/input/by-path/pci-0000:00:14.0-usb-0:3:1.0-event"));
    }

    #[test]
    fn extract_frontend_from_empty_vector() {
        assert_eq!(extract_frontend_from_paths(&vec![]).len(), 0);
    }

    #[test]
    fn extract_frontend_from_one_item_vector() {
        let result = extract_frontend_from_paths(&vec![PathBuf::from("/sys/class/rc/rc0/device/dvb/dvb0.frontend0")]);
        assert_eq!(result.len(), 1);
        assert_eq!(*result.get(0).unwrap(), FrontendId{adapter: 0, frontend: 0});
    }

    #[test]
    fn extract_frontend_from_two_item_vector() {
        let result = extract_frontend_from_paths(&vec![
            PathBuf::from("/sys/class/rc/rc0/device/dvb/dvb0.frontend0"),
            PathBuf::from("/sys/class/rc/rc0/device/dvb/dvb1.frontend0")
        ]);
        assert_eq!(result.len(), 2);
        assert_eq!(*result.get(0).unwrap(), FrontendId{adapter: 0, frontend: 0});
        assert_eq!(*result.get(1).unwrap(), FrontendId{adapter: 1, frontend: 0});
    }
}


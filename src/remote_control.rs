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
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use futures::channel::mpsc::Sender;
use glob::glob;
use libc;
use notify::{Watcher, RecursiveMode, RawEvent, op, raw_watcher};
use regex::Regex;

use control_window::Message;
use frontend_manager::FrontendId;
use input_event_codes;

#[derive(Debug)]
pub struct RemoteControl {
    pub frontend_ids: Vec<FrontendId>,
    pub lirc_path: PathBuf,
    pub sys_rc_path: PathBuf,  // Cache this even though it is refindable.
    pub device_event_path: PathBuf,  // Cache this even though it is refindable.
    pub device_file: File,
}

lazy_static! {
static ref REMOTES: Mutex<Vec<Arc<RemoteControl>>> = Mutex::new(vec![]);
}

/// Given a /dev/lircX path return the appropriate /sys/class/rc/rcY path.
fn get_sys_path_from_lirc_path(lirc_path: &PathBuf) -> Option<PathBuf> {
    let rc_devices_lirc_paths = match glob::glob("/sys/class/rc/rc*/lirc*") {
        Ok(paths) => paths.map(|x| x.unwrap()).collect::<Vec<PathBuf>>(),
        Err(e) => panic!("Glob failure: {}", e),
    };
    let rc_paths = rc_devices_lirc_paths.iter()
        .filter(|pb| pb.file_name() == lirc_path.file_name())
        .collect::<Vec<&PathBuf>>();
    if rc_paths.len() == 0 { None }
    else {
        assert_eq!(rc_paths.len(), 1);
        let mut rv = rc_paths[0].to_path_buf();
        assert!(rv.pop());
        Some(rv)
    }
}

/// Create an /dev/inputs/eventsX `PathBuf` from the /sys/class/rc/rcY `PathBuf`.
///
/// This has been constructed from the data observed on Debian Sid.
/// It is assumed that all Linux post 4.6 will be the same.
fn create_event_path_from_sys_path(path: &PathBuf) -> PathBuf {
    let components = path.components().map(|x| x.as_os_str().to_str().unwrap()).collect::<Vec<&str>>();
    assert_eq!(components[0], "..");
    assert_eq!(components[1], "..");
    assert_eq!(components[components.len() -2], "rc");
    let mut event_path_string = String::from("/dev/input/by-path/pci-");
    event_path_string += components[4];
    event_path_string += "-usb-0:";
    event_path_string += components[components.len() - 3].split("-").collect::<Vec<&str>>()[1]; // TODO Seems overcomplicated.
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

ioctl_write_int!(ioctl_eviocgrab, b'E', 0x90);

impl RemoteControl {
    fn new(lirc_path: &PathBuf) -> RemoteControl {
        let sys_rc_path = get_sys_path_from_lirc_path(lirc_path).unwrap(); // TODO Is it certain this will not fail?
        let frontend_ids = find_frontends_for_remote_control(&sys_rc_path);
        let device_event_path= match sys_rc_path.read_link() {
            Ok(path) => create_event_path_from_sys_path(&path),
            Err(e) => panic!("Could not read symbolic link for remote control: {}", e),
        };
        while ! device_event_path.exists() {
            // TODO Need to avoid an infinite loop here.
            thread::sleep(Duration::from_millis(500));
        }
        let device_file = OpenOptions::new()
            .read(true)
            .open(&device_event_path)
            .expect(&format!("Cannot open the event stream {}", device_event_path.to_str().unwrap()));
        unsafe {
            ioctl_eviocgrab(device_file.as_raw_fd(), 1).unwrap();
        }
        RemoteControl {
            frontend_ids,
            lirc_path: lirc_path.to_path_buf(),
            sys_rc_path: sys_rc_path.to_path_buf(),
            device_event_path,
            device_file,
        }
    }
}

/// A keystroke intended for a given frontend for use in sending messages between the
/// remote controller daemon and the GUI.
#[derive(Debug)]
pub struct TargettedKeystroke {
    pub frontend_id: FrontendId, // Used in control_window
    pub keystroke: u32, // Used in control_window
    pub value: u32, // Used in control_window
}

/// Print all the events currently available on the event special file.
fn process_events_for_device(remote_control: &Arc<RemoteControl>, to_cw: &mut Sender<Message>) {
    // TODO is it reasonable to assume less than 64 events?
    let buffer = [libc::input_event{time: libc::timeval{tv_sec: 0, tv_usec: 0}, type_: 0, code: 0, value: 0}; 64];
    let item_size = std::mem::size_of::<libc::input_event>();
    let rc = unsafe {
        libc::read(remote_control.device_file.as_raw_fd(), buffer.as_ptr() as *mut libc::c_void, item_size * 64)
    };
    if rc < 0 { panic!("Read failed:"); }
    let event_count = rc as usize /  item_size;
    assert_eq!(item_size * event_count, rc as usize);
    for i in 0 .. event_count {
        let item = buffer[i];
        if item.type_ == input_event_codes::EV_KEY as u16 {
            to_cw.try_send(Message::TargettedKeystrokeReceived{
                tk: TargettedKeystroke{frontend_id: remote_control.frontend_ids[0].clone(), keystroke: item.code as u32, value: item.value as u32},
            }).unwrap();
        }
    }
}

pub fn rc_event_listener(mut to_cw: Sender<Message>) {
    loop {
        // TODO What happens if a new adapter is inserted or an existing remote removed
        //   before a remote control event happens.
        let remote_controls = match REMOTES.lock() {
            Ok(data) => data.iter().map(|x| x.clone()).collect::<Vec<Arc<RemoteControl>>>(),
            Err(_) => vec![],
        };
        let mut pollfds = remote_controls.iter().map(|device| {
            libc::pollfd{fd: device.device_file.as_raw_fd(), events: libc::POLLIN, revents: 0}
        }).collect::<Vec<libc::pollfd>>();
        if pollfds.len() > 0 {
            unsafe {
                // TODO Switch this to not being fully blocking but instead to have a timeout to allow a remote control refresh?
                let count = libc::poll(pollfds.as_mut_ptr(), pollfds.len() as u64, -1);
                assert!(count > 0);
                for i in 0..pollfds.len() {
                    if pollfds[i].revents != 0 {
                        process_events_for_device(&remote_controls[i], &mut to_cw);
                    }
                }
            }
        }
    }
}

fn print_all_remotes_known() {
    match REMOTES.lock() {
        Ok(data) => {
            println!("Current RemoteControls:");
            for item in data.iter() {
                println!("\t{:?}", item);
            }
        },
        Err(_) => println!("No data"),
    }
}

/// Check for all the remote controls already known to the system and add then to the collection
/// of known remote controls.
fn add_already_installed_remotes() {
    let lirc_devices = match glob::glob("/dev/lirc*") {
        Ok(paths) => paths.map(|x| x.unwrap()).collect::<Vec<PathBuf>>(),
        Err(e) => panic!("Glob failure: {}", e),
    };
    if  lirc_devices.is_empty() { return; };
    match REMOTES.lock () {
        Ok(mut data) => {
            for rc in lirc_devices.iter()
                .filter(|lirc_path| get_sys_path_from_lirc_path(lirc_path).is_some())
                .map(|lirc_path| Arc::new(RemoteControl::new(lirc_path))) {
                data.push(rc);
            }
        },
        Err(_) => panic!("Couldn't lock REMOTES for addition. ")
    };
    print_all_remotes_known();
}

fn add_appeared_remote_control(lirc_path: PathBuf) {
    println!("LIRC appeared: {:?}", &lirc_path);
    // TODO is a delay required here to ensure the /sys filestore has been updated
    //   on the presence of the /dev/lircX
    if get_sys_path_from_lirc_path(&lirc_path).is_some() {
        match REMOTES.lock() {
            Ok(mut data) => {
                data.push(Arc::new(RemoteControl::new(&lirc_path)))
            },
            Err(_) => panic!("Failed to lock REMOTES for addition."),
        }
    }
    print_all_remotes_known();
}

fn remove_disappeared_remote_control(lirc_path: PathBuf) {
    println!("LIRC disappeared: {:?}", &lirc_path);
    match REMOTES.lock() {
        Ok(mut data) => {
            //  TODO ensure that this properly tidies up all the things such as EVIOCGRAB.
            data.retain(|d| d.lirc_path != lirc_path)
        },
        Err(_) => panic!("Failed to lock REMOTES for removal."),
    };
    print_all_remotes_known();
}

pub fn run(mut to_cw: Sender<Message>) {
    add_already_installed_remotes();
    thread::spawn(|| rc_event_listener(to_cw));
    let (transmit_end, receive_end) = channel();
    let mut watcher = raw_watcher(transmit_end).unwrap();
    watcher.watch("/dev", RecursiveMode::NonRecursive).unwrap();
    loop {
        match receive_end.recv() {
            Ok(RawEvent { path: Some(path), op: Ok(op), cookie: _cookie }) => {
                match op {
                    op::CREATE => {
                        if path.file_name().unwrap().to_str().unwrap().starts_with("lirc") {
                            add_appeared_remote_control(path);
                        }
                    },
                    op::REMOVE => {
                        if path.file_name().unwrap().to_str().unwrap().starts_with("lirc") {
                            remove_disappeared_remote_control(path);
                        }
                    },
                    _ => {},
                }
            },
            Ok(event) => println!("remote_control::run: broken event: {:?}", event),
            Err(e) => println!("remote_control::run: watch error: {:?}", e),
        }
    }
    println!("remote_control::run terminated.");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rc0_on_lynet_debian_linux() {
        assert_eq!(
            create_event_path_from_sys_path(&PathBuf::from("../../devices/pci0000:00/0000:00:14.0/usb2/2-1/2-1:1.0/rc/rc0")),
            PathBuf::from("/dev/input/by-path/pci-0000:00:14.0-usb-0:1:1.0-event"));
    }

    #[test]
    fn rc0_on_anglides_debian_linux() {
        assert_eq!(
            create_event_path_from_sys_path(&PathBuf::from("../../devices/pci0000:00/0000:00:1d.7/usb4/4-5/4-5.2/4-5.2.4/4-5.2.4.1/4-5.2.4.1.1/4-5.2.4.1.1:1.0/rc/rc0")),
            PathBuf::from("/dev/input/by-path/pci-0000:00:1d.7-usb-0:5.2.4.1.1:1.0-event"));
    }

    #[test]
    fn rc1_on_lynet_debian_linux() {
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


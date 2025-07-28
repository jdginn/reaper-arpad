use fragile::Fragile;
use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::ProjectContext::CurrentProject;
use reaper_medium::{ControlSurface, MediaTrack, Reaper, ReaperSession, TrackAttributeKey};
use std::error::Error;
use std::sync::OnceLock;

use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};

use crossbeam_channel::{bounded, Receiver, Sender};
use std::thread;

fn guid_to_string(guid: reaper_low::raw::GUID) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        guid.Data1,
        guid.Data2,
        guid.Data3,
        guid.Data4[0],
        guid.Data4[1],
        guid.Data4[2],
        guid.Data4[3],
        guid.Data4[4],
        guid.Data4[5],
        guid.Data4[6],
        guid.Data4[7],
    )
}

fn get_track_idx(reaper: &Reaper, track: MediaTrack) -> u32 {
    unsafe {
        return reaper.get_media_track_info_value(track, TrackAttributeKey::TrackNumber) as u32;
    }
}

fn get_track_guid(reaper: &Reaper, track: MediaTrack) -> String {
    unsafe {
        let track_id = reaper.get_set_media_track_info_get_guid(track);
        return guid_to_string(track_id);
    }
}

fn get_track_by_guid(reaper: &Reaper, guid: &str) -> Option<MediaTrack> {
    let master_track = reaper.get_master_track(CurrentProject);
    if get_track_guid(&reaper, master_track) == guid {
        return Some(master_track);
    }
    for i in 0..reaper.count_tracks(CurrentProject) {
        let track = reaper.get_track(CurrentProject, i).unwrap();
        if get_track_guid(&reaper, track) == guid {
            return Some(track);
        }
    }
    return None;
}

#[derive(Debug)]
struct ArpadSurface {
    osc_sender: Sender<OscPacket>,
    sock: UdpSocket,
    reaper: Reaper,
}

impl ControlSurface for ArpadSurface {
    fn set_track_list_change(&self) {
        for i in 0..self.reaper.count_tracks(CurrentProject) {
            let track = self.reaper.get_track(CurrentProject, i).unwrap();
            let track_idx = get_track_idx(&self.reaper, track);
            let track_guid = get_track_guid(&self.reaper, track);
            self.osc_sender
                .send(OscPacket::Message(OscMessage {
                    addr: format!("/track/{}/index", track_guid).to_string(),
                    args: vec![OscType::Int(track_idx as i32)],
                }))
                .unwrap();
            unsafe {
                for i in 0..self
                    .reaper
                    .get_track_num_sends(track, reaper_medium::TrackSendCategory::Send)
                {
                    let dest = self
                        .reaper
                        .get_track_send_info_desttrack(
                            track,
                            reaper_medium::TrackSendDirection::Send,
                            i,
                        )
                        .unwrap();
                    self.osc_sender
                        .send(OscPacket::Message(OscMessage {
                            addr: format!("/track/{}/send/{}", track_guid, i).to_string(),
                            args: vec![OscType::String(get_track_guid(&self.reaper, dest))],
                        }))
                        .unwrap();
                }
            }
        }
    }
    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        let track_guid = get_track_guid(&self.reaper, args.track);
        self.osc_sender
            .send(OscPacket::Message(OscMessage {
                addr: format!("/track/{}/volume", track_guid).to_string(),
                args: vec![OscType::Float(args.volume.into_inner() as f32)],
            }))
            .unwrap();
    }
    fn set_surface_pan(&self, args: reaper_medium::SetSurfacePanArgs) {
        let track_guid = get_track_guid(&self.reaper, args.track);
        self.osc_sender
            .send(OscPacket::Message(OscMessage {
                addr: format!("/track/{}/pan", track_guid).to_string(),
                args: vec![OscType::Float(args.pan.into_inner() as f32)],
            }))
            .unwrap();
    }
    fn set_surface_mute(&self, args: reaper_medium::SetSurfaceMuteArgs) {
        let track_guid = get_track_guid(&self.reaper, args.track);
        self.osc_sender
            .send(OscPacket::Message(OscMessage {
                addr: format!("/track/{}/mute", track_guid).to_string(),
                args: vec![OscType::Int(if args.is_mute { 1 } else { 0 })],
            }))
            .unwrap();
    }
    fn run(&mut self) {
        let mut buf = [0u8; rosc::decoder::MTU];
        loop {
            match self.sock.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    if let Ok((_addr, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                        handle_packet(self.reaper.clone(), packet);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, exit loop
                    break;
                }
                Err(e) => {
                    eprintln!("OSC receive error: {:?}", e);
                    break;
                }
            }
        }
    }
}

// Spawn the OSC sending thread
fn start_sender_thread(dev_addr: SocketAddrV4, sock: UdpSocket, osc_receiver: Receiver<OscPacket>) {
    thread::spawn(move || {
        for msg in osc_receiver.iter() {
            if let Ok(buf) = encoder::encode(&msg) {
                let _ = sock.send_to(buf.as_slice(), dev_addr);
            }
        }
    });
}

fn parse_osc_address(addr: &str) -> Vec<&str> {
    addr.split('/').filter(|s| !s.is_empty()).collect()
}

fn handle_packet(reaper: Reaper, packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            let segments = parse_osc_address(&msg.addr);
            match segments.as_slice() {
                ["track", guid, "volume"] => match get_track_by_guid(&reaper, guid) {
                    Some(track) => unsafe {
                        reaper
                            .set_media_track_info_value(
                                track,
                                TrackAttributeKey::Vol,
                                msg.args[0].clone().float().unwrap() as f64,
                            )
                            .unwrap();
                    },
                    None => {
                        println!("Track not found: {}", guid);
                    }
                },
                ["track", guid, "pan"] => match get_track_by_guid(&reaper, guid) {
                    Some(track) => unsafe {
                        reaper
                            .set_media_track_info_value(
                                track,
                                TrackAttributeKey::Pan,
                                msg.args[0].clone().float().unwrap() as f64,
                            )
                            .unwrap();
                    },
                    None => {
                        println!("Track not found: {}", guid);
                    }
                },
                _ => {}
            }
            println!("OSC message: {:?}", msg);
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC bundle: {:?}", bundle);
        }
    }
}

const HOST_ADDR: &str = "0.0.0.0:9090";
const DEVICE_ADDR: &str = "0.0.0.0:9091";

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
        SocketAddrV4::from_str(arg).unwrap()
    }
    let host_addr = get_addr_from_arg(HOST_ADDR);
    let dev_addr = get_addr_from_arg(DEVICE_ADDR);
    let sock = UdpSocket::bind(host_addr).unwrap();
    sock.set_nonblocking(true)?;
    let (osc_sender, osc_receiver) = bounded(128); // buffer size as needed
    start_sender_thread(dev_addr, sock.try_clone().unwrap(), osc_receiver);

    let mut session = reaper_medium::ReaperSession::load(context);
    let reaper = session.reaper().clone();
    let mut arpad = ArpadSurface {
        sock,
        osc_sender,
        reaper: reaper.clone(),
    };
    arpad.run();
    match session.plugin_register_add_csurf_inst(Box::new(arpad)) {
        Ok(_) => {}
        Err(_) => {
            println!("Failed to load csurf");
        }
    }
    let _ = REAPER_SESSION.set(Fragile::new(session));

    Ok(())
}
static REAPER_SESSION: OnceLock<Fragile<ReaperSession>> = OnceLock::new();

use fragile::Fragile;
use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::ProjectContext::CurrentProject;
use reaper_medium::{ControlSurface, Reaper, ReaperSession, TrackAttributeKey};
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

fn get_id_guid(reaper: &Reaper, track: reaper_medium::MediaTrack) -> (f64, String) {
    unsafe {
        let track_num = reaper.get_media_track_info_value(track, TrackAttributeKey::TrackNumber);
        let track_id = reaper.get_set_media_track_info_get_guid(track);
        return (track_num, guid_to_string(track_id));
    }
}

#[derive(Debug)]
struct ArpadSurface {
    osc_sender: Sender<OscPacket>,
    reaper: Reaper,
}

impl ControlSurface for ArpadSurface {
    fn set_track_list_change(&self) {
        for i in 0..self.reaper.count_tracks(CurrentProject) {
            let track = self.reaper.get_track(CurrentProject, i).unwrap();
            let (track_num, track_id) = get_id_guid(&self.reaper, track);
            self.osc_sender
                .send(OscPacket::Message(OscMessage {
                    addr: format!("/track/{}/index", track_id).to_string(),
                    args: vec![OscType::Int(track_num as i32)],
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
                    let (_, dest_id) = get_id_guid(&self.reaper, dest);
                    self.osc_sender
                        .send(OscPacket::Message(OscMessage {
                            addr: format!("/track/{}/send/{}", track_id, i).to_string(),
                            args: vec![OscType::String(dest_id)],
                        }))
                        .unwrap();
                }
            }
        }
    }
    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        let (_, track_id) = get_id_guid(&self.reaper, args.track);
        self.osc_sender
            .send(OscPacket::Message(OscMessage {
                addr: format!("/track/{}/volume", track_id).to_string(),
                args: vec![OscType::Float(args.volume.into_inner() as f32)],
            }))
            .unwrap();
    }
    fn set_surface_pan(&self, args: reaper_medium::SetSurfacePanArgs) {
        let (_, track_id) = get_id_guid(&self.reaper, args.track);
        self.osc_sender
            .send(OscPacket::Message(OscMessage {
                addr: format!("/track/{}/pan", track_id).to_string(),
                args: vec![OscType::Float(args.pan.into_inner() as f32)],
            }))
            .unwrap();
    }
    fn set_surface_mute(&self, args: reaper_medium::SetSurfaceMuteArgs) {
        let (_, track_id) = get_id_guid(&self.reaper, args.track);
        self.osc_sender
            .send(OscPacket::Message(OscMessage {
                addr: format!("/track/{}/mute", track_id).to_string(),
                args: vec![OscType::Int(if args.is_mute { 1 } else { 0 })],
            }))
            .unwrap();
    }
}

fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
    SocketAddrV4::from_str(arg).unwrap()
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

const HOST_ADDR: &str = "0.0.0.0:9090";
const DEVICE_ADDR: &str = "0.0.0.0:9091";

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let host_addr = get_addr_from_arg(HOST_ADDR);
    let dev_addr = get_addr_from_arg(DEVICE_ADDR);
    let sock = UdpSocket::bind(host_addr).unwrap();
    let (osc_sender, osc_receiver) = bounded(128); // buffer size as needed
    start_sender_thread(dev_addr, sock.try_clone().unwrap(), osc_receiver);

    let mut session = reaper_medium::ReaperSession::load(context);
    let reaper = session.reaper();
    let mut arpad = ArpadSurface {
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

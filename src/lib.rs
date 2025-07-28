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
    dev_addr: SocketAddrV4,
    sock: UdpSocket,
    reaper: Reaper,
}

impl ControlSurface for ArpadSurface {
    fn set_track_list_change(&self) {
        for i in 0..self.reaper.count_tracks(CurrentProject) {
            let track = self.reaper.get_track(CurrentProject, i).unwrap();
            let (track_num, track_id) = get_id_guid(&self.reaper, track);
            let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                addr: format!("/track/{}/index", track_id).to_string(),
                args: vec![OscType::Int(track_num as i32)],
            }))
            .unwrap();
            self.sock
                .send_to(msg_buf.as_slice(), self.dev_addr)
                .unwrap();
        }
    }
    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        let (_, track_id) = get_id_guid(&self.reaper, args.track);
        let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: format!("/track/{}/volume", track_id).to_string(),
            args: vec![OscType::Float(args.volume.into_inner() as f32)],
        }))
        .unwrap();
        self.sock
            .send_to(msg_buf.as_slice(), self.dev_addr)
            .unwrap();
    }
}

fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
    SocketAddrV4::from_str(arg).unwrap()
}

const HOST_ADDR: &str = "0.0.0.0:9090";
const DEVICE_ADDR: &str = "0.0.0.0:9091";

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let host_addr = get_addr_from_arg(HOST_ADDR);
    let dev_addr = get_addr_from_arg(DEVICE_ADDR);
    let sock = UdpSocket::bind(host_addr).unwrap();

    let mut session = reaper_medium::ReaperSession::load(context);
    let reaper = session.reaper();
    println!("Got reaper");
    let mut arpad = ArpadSurface {
        sock,
        dev_addr,
        reaper: reaper.clone(),
    };
    arpad.run();
    match session.plugin_register_add_csurf_inst(Box::new(arpad)) {
        Ok(_) => {
            println!("Loaded csurf");
        }
        Err(_) => {
            println!("Failed to load csurf");
        }
    }
    let _ = REAPER_SESSION.set(Fragile::new(session));

    Ok(())
}
static REAPER_SESSION: OnceLock<Fragile<ReaperSession>> = OnceLock::new();

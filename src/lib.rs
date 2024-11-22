use fragile::Fragile;
use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{ControlSurface, ReaperSession};
use std::error::Error;
use std::sync::OnceLock;

// static MANAGER: TrackSetManager = TrackSetManager { stack: Vec::new() };

#[derive(Debug)]
struct ArpadSurface {}

impl ControlSurface for ArpadSurface {
    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        let _ = args;
        println!("Set surface volume");
    }
    fn set_track_list_change(&self) {
        println!("Track list change");
    }
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let mut session = reaper_medium::ReaperSession::load(context);
    println!("Got reaper");
    let mut arpad = ArpadSurface {};
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

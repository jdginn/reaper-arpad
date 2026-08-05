use std::collections::HashMap;

use reaper_medium::ProjectContext::CurrentProject;
use reaper_medium::Reaper;

use crossbeam_channel::Sender;
use rosc::OscPacket;

use crate::track_routes::{self};
use crate::utils::get_track_guid;
use crate::OscRoute;

#[derive(Debug)]
pub enum PollError {
    Reaper(reaper_medium::ReaperFunctionError),
    Send(crossbeam_channel::SendError<OscPacket>),
}

pub struct PollManager {
    sources: Vec<Box<dyn PollSource>>,
}

impl Default for PollManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PollManager {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: Box<dyn PollSource>) {
        self.sources.push(source);
    }

    /// Called in the main run loop
    pub fn poll_all(&mut self, osc_sender: &Sender<OscPacket>) {
        for source in self.sources.iter_mut() {
            source
                .poll_and_send(osc_sender)
                .map_err(|e| {
                    eprintln!("Polling error: {:?}", e);
                })
                .unwrap_or(());
        }
    }
}

// Trait for anything that can be polled for feedback to send via OSC
pub trait PollSource {
    /// Called periodically to check for changes and send feedback
    /// Returns true if feedback was sent
    fn poll_and_send(&mut self, osc_sender: &Sender<OscPacket>) -> Result<(), PollError>;
}

pub struct TrackColorPollSource {
    reaper: Reaper,
    prev_colors: HashMap<String, reaper_medium::RgbColor>,
}

impl TrackColorPollSource {
    pub fn new(reaper: Reaper) -> Self {
        Self {
            reaper,
            prev_colors: HashMap::new(),
        }
    }
}

impl PollSource for TrackColorPollSource {
    fn poll_and_send(&mut self, osc_sender: &Sender<OscPacket>) -> Result<(), PollError> {
        for i in 0..self.reaper.count_tracks(CurrentProject) {
            let track = self.reaper.get_track(CurrentProject, i).unwrap();
            let guid = get_track_guid(&self.reaper, track);
            let color =
                unsafe { self.reaper.get_set_media_track_info_get_custom_color(track) }.color;
            let rgb_color = self.reaper.color_from_native(color);
            if let Some(prev_color) = self.prev_colors.get(&guid) {
                if *prev_color != rgb_color {
                    self.prev_colors.insert(guid.clone(), rgb_color);
                    track_routes::TrackColorRoute::build_packets(
                        track_routes::TrackColorArgs {
                            track,
                            r: rgb_color.r,
                            g: rgb_color.g,
                            b: rgb_color.b,
                        },
                        &self.reaper,
                    )
                    .into_iter()
                    .for_each(|packet| {
                        osc_sender
                            .send(packet)
                            .map_err(PollError::Send)
                            .unwrap_or(());
                    });
                }
            } else {
                self.prev_colors.insert(guid.clone(), rgb_color);
                track_routes::TrackColorRoute::build_packets(
                    track_routes::TrackColorArgs {
                        track,
                        r: rgb_color.r,
                        g: rgb_color.g,
                        b: rgb_color.b,
                    },
                    &self.reaper,
                )
                .into_iter()
                .for_each(|packet| {
                    osc_sender
                        .send(packet)
                        .map_err(PollError::Send)
                        .unwrap_or(());
                });
            }
        }
        Ok(())
    }
}

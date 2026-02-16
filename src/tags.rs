use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

pub struct TrackMetadata {
    pub artist: String,
    pub album: String,
    pub title: Option<String>,
    pub track_number: Option<u32>,
}

pub fn read_tags(path: &Path) -> Option<TrackMetadata> {
    let tagged_file = Probe::open(path).ok()?.read().ok()?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())?;

    let artist = tag.artist()?.to_string();
    let album = tag.album()?.to_string();

    if artist.is_empty() || album.is_empty() {
        return None;
    }

    let title = tag.title().map(|t| t.to_string()).filter(|t| !t.is_empty());
    let track_number = tag.track();

    Some(TrackMetadata {
        artist,
        album,
        title,
        track_number,
    })
}

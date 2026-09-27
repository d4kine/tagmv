use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
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

    // Album artist keeps compilations in one folder; per-track artist is the fallback.
    let artist = tag
        .get_string(&ItemKey::AlbumArtist)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
        .or_else(|| tag.artist().map(|s| s.to_string()))?;
    let album = tag.album()?.to_string();

    if artist.is_empty() || album.is_empty() {
        return None;
    }

    let title = tag.title().map(|t| t.to_string()).filter(|t| !t.is_empty());
    let track_number = tag.track().filter(|&n| n > 0);

    Some(TrackMetadata {
        artist,
        album,
        title,
        track_number,
    })
}

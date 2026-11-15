//! Metadata tagging for downloaded audio files using `mp4ameta`.

use std::path::Path;

pub fn tag_audio_file(
    path: &Path,
    title: &str,
    artist: &str,
    album: Option<&str>,
    cover: Option<&[u8]>,
) -> anyhow::Result<()> {
    let mut tag = mp4ameta::Tag::read_from_path(path)?;
    tag.set_title(title);

    let artists: Vec<&str> = artist.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    for (i, a) in artists.iter().enumerate() {
        if i == 0 {
            tag.set_artist(*a);
        } else {
            tag.add_artist(*a);
        }
    }
    if let Some(a) = artists.first() {
        tag.set_album_artist(*a);
    }
    if let Some(alb) = album {
        tag.set_album(alb);
    }
    if let Some(c) = cover {
        let img = match c.get(..4) {
            Some([0xff, 0xd8, ..]) => Some(mp4ameta::Img::jpeg(c.to_vec())),
            Some([0x89, b'P', b'N', b'G']) => Some(mp4ameta::Img::png(c.to_vec())),
            _ => None,
        };
        if let Some(artwork) = img {
            tag.set_artwork(artwork);
        }
    }
    tag.write_to_path(path)?;
    Ok(())
}

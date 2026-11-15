//! Dolby Atmos & DASH manifest engine ported from BitChord-PC (src/dash.rs).
//!
//! Parses DASH MPD manifests for Dolby (E-)AC-3 JOC renditions, resolves segment
//! templates, and provides stream metadata.

use std::time::Duration;
use roxmltree::Document;

pub const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug, PartialEq)]
pub struct DolbyPlan {
    pub init: String,
    pub segments: Vec<String>,
    pub bandwidth: u32,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub is_dolby_atmos: bool,
    pub codec: String,
}

impl DolbyPlan {
    pub fn parse(xml: &str, mpd_url: &str) -> anyhow::Result<Self> {
        let doc = Document::parse(xml)?;
        let mpd = doc.root_element();
        anyhow::ensure!(mpd.tag_name().name() == "MPD", "not a DASH manifest");
        let base = url::Url::parse(mpd_url)?;
        let period = mpd.children().find(|n| n.has_tag_name("Period")).ok_or_else(|| anyhow::anyhow!("manifest has no Period"))?;
        let period_ms = duration_attr(mpd.attribute("mediaPresentationDuration"))
            .or_else(|| duration_attr(period.attribute("duration")))
            .unwrap_or(0);

        let mut reps: Vec<roxmltree::Node> = period.children()
            .filter(|n| n.has_tag_name("AdaptationSet"))
            .flat_map(|a| a.children().filter(|n| n.has_tag_name("Representation")).map(move |r| (a, r)))
            .filter(|(a, r)| {
                let mime = r.attribute("mimeType").or_else(|| a.attribute("mimeType")).unwrap_or("");
                mime.is_empty() || mime.starts_with("audio/")
            })
            .map(|(_, r)| r)
            .collect();

        reps.sort_by_key(|r| match r.attribute("codecs").unwrap_or("") {
            c if c.contains("ec-3") => 0,
            c if c.contains("ac-3") => 1,
            c if c.contains("flac") => 2,
            _ => 3,
        });

        let rep = reps.iter().find(|r| {
            r.attribute("codecs").is_some_and(|c| c.contains("ec-3") || c.contains("ac-3"))
        }).or_else(|| reps.first())
          .copied()
          .ok_or_else(|| anyhow::anyhow!("manifest has no audio rendition"))?;

        let bandwidth = rep.attribute("bandwidth").and_then(|v| v.parse().ok()).unwrap_or(0);

        fn up<'a, 'i>(n: roxmltree::Node<'a, 'i>, tag: &str) -> Option<roxmltree::Node<'a, 'i>> {
            let mut cur = Some(n);
            while let Some(c) = cur {
                if let Some(found) = c.children().find(|k| k.has_tag_name(tag)) {
                    return Some(found);
                }
                cur = c.parent();
            }
            None
        }

        let template = up(rep, "SegmentTemplate").ok_or_else(|| anyhow::anyhow!("rendition has no SegmentTemplate"))?;
        let base = {
            let mut cur = Some(rep);
            let mut found = None;
            while let Some(c) = cur {
                if let Some(b) = c.children().find(|k| k.has_tag_name("BaseURL")).and_then(|b| b.text()) {
                    found = Some(b.trim().to_owned());
                    break;
                }
                cur = c.parent();
            }
            match found {
                Some(b) => base.join(&b)?,
                None => base,
            }
        };

        let address = |pattern: &str, number: u64, time: u64| -> anyhow::Result<String> {
            let mut s = pattern.replace("$RepresentationID$", rep.attribute("id").unwrap_or(""));
            s = s.replace("$Bandwidth$", &bandwidth.to_string());
            s = s.replace("$Time$", &time.to_string());
            while let Some(start) = s.find("$Number") {
                let end = s[start + 1..].find('$').map(|i| start + i + 2).ok_or_else(|| anyhow::anyhow!("unclosed $Number$"))?;
                let token = &s[start + 1..end - 1];
                let width = token.split('%').nth(1).and_then(|w| w.trim_start_matches('0').trim_end_matches('d').parse::<usize>().ok());
                let text = match width {
                    Some(w) => format!("{number:0w$}"),
                    None => number.to_string(),
                };
                s.replace_range(start..end, &text);
            }
            Ok(if url::Url::parse(&s).is_ok() { s } else { base.join(&s)?.into() })
        };

        let init = address(template.attribute("initialization").ok_or_else(|| anyhow::anyhow!("no initialization"))?, 0, 0)?;
        let media = template.attribute("media").ok_or_else(|| anyhow::anyhow!("no media"))?;
        let scale = template.attribute("timescale").and_then(|v| v.parse::<u64>().ok()).unwrap_or(1);
        let start = template.attribute("startNumber").and_then(|v| v.parse::<u64>().ok()).unwrap_or(1);

        let mut segments = Vec::new();
        if let Some(timeline) = template.children().find(|k| k.has_tag_name("SegmentTimeline")) {
            let mut time = 0u64;
            let mut seq = start;
            for s in timeline.children().filter(|k| k.has_tag_name("S")) {
                if let Some(t) = s.attribute("t").and_then(|v| v.parse().ok()) {
                    time = t;
                }
                let count = 1 + s.attribute("r").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0).max(0) as u64;
                let step = s.attribute("d").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                for _ in 0..count {
                    segments.push(address(media, seq, time)?);
                    seq += 1;
                    time += step;
                }
            }
        } else if let Some(duration) = template.attribute("duration").and_then(|v| v.parse::<u64>().ok()) {
            let total = (period_ms as f64 / 1000.0 * scale as f64 / duration as f64).ceil() as u64;
            for i in 0..total {
                segments.push(address(media, start + i, i * duration)?);
            }
        }

        let codecs = rep.attribute("codecs").unwrap_or("");
        let is_dolby_atmos = codecs.contains("ec-3") || codecs.contains("ac-3") || codecs.contains("joc");
        let sample_rate = rep.attribute("audioSamplingRate").and_then(|v| v.parse().ok()).unwrap_or(48000);

        Ok(Self {
            init,
            segments,
            bandwidth,
            duration_ms: period_ms,
            sample_rate,
            is_dolby_atmos,
            codec: codecs.to_string(),
        })
    }
}

fn duration_attr(attr: Option<&str>) -> Option<u64> {
    let s = attr?;
    let mut total = 0.0f64;
    let mut buf = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_digit() || c == '.' {
            buf.push(c);
        } else {
            if let Ok(v) = buf.parse::<f64>() {
                match c {
                    'H' => total += v * 3600.0,
                    'M' => total += v * 60.0,
                    'S' => total += v,
                    _ => {}
                }
            }
            buf.clear();
        }
        i += 1;
    }
    Some((total * 1000.0) as u64)
}

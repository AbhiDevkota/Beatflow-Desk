//! Lyrics parsing engine for standard LRC, enhanced LRC, and Apple Music TTML.

use roxmltree::Document;
use super::model::{LyricLine, ParsedLyrics, TimedRun};

/// Parses standard and enhanced LRC text.
pub fn parse_lrc(input: &str) -> ParsedLyrics {
    let mut lines = Vec::new();
    let mut has_word_sync = false;

    for raw_line in input.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("[ti:") || trimmed.starts_with("[ar:") || trimmed.starts_with("[al:") || trimmed.starts_with("[by:") || trimmed.starts_with("[offset:") {
            continue;
        }

        // Match [mm:ss.xx] or [mm:ss:xx] or [mm:ss]
        if let Some(close_bracket) = trimmed.find(']')
            && trimmed.starts_with('[') && close_bracket > 1 {
                let time_str = &trimmed[1..close_bracket];
                if let Some(start_ms) = parse_lrc_timestamp(time_str) {
                    let text = trimmed[close_bracket + 1..].trim();
                    let (clean_text, runs) = parse_enhanced_runs(text, start_ms);
                    if !runs.is_empty() {
                        has_word_sync = true;
                    }

                    lines.push(LyricLine {
                        text: clean_text,
                        start_ms,
                        end_ms: start_ms + 4000, // will be adjusted relative to next line
                        runs,
                        is_instrumental_gap: false,
                    });
                }
            }
    }

    // Sort by start_ms
    lines.sort_by_key(|l| l.start_ms);

    // Compute end_ms from next line's start_ms
    let len = lines.len();
    for i in 0..len {
        if i + 1 < len {
            let next_start = lines[i + 1].start_ms;
            if next_start > lines[i].start_ms {
                lines[i].end_ms = next_start;
            }
        }
    }

    let is_synced = !lines.is_empty();

    // If unsynced, convert whole lines to plain lines
    if !is_synced {
        for l in input.lines() {
            let t = l.trim();
            if !t.is_empty() {
                lines.push(LyricLine {
                    text: t.to_string(),
                    start_ms: 0,
                    end_ms: 0,
                    runs: Vec::new(),
                    is_instrumental_gap: false,
                });
            }
        }
    }

    ParsedLyrics {
        lines,
        is_synced,
        has_word_sync,
        raw_ttml: None,
    }
}

fn parse_lrc_timestamp(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    let mins: u64 = parts[0].parse().ok()?;
    let sec_parts: Vec<&str> = parts[1].split(['.', ':']).collect();
    let secs: u64 = sec_parts[0].parse().ok()?;
    let millis: u64 = if sec_parts.len() > 1 {
        let frac = sec_parts[1];
        if frac.len() == 2 {
            frac.parse::<u64>().ok()? * 10
        } else if frac.len() >= 3 {
            frac[..3].parse::<u64>().ok()?
        } else {
            frac.parse::<u64>().ok()? * 100
        }
    } else {
        0
    };

    Some(mins * 60_000 + secs * 1_000 + millis)
}

fn parse_enhanced_runs(text: &str, line_start_ms: u64) -> (String, Vec<TimedRun>) {
    let mut clean_text = String::new();
    let mut runs = Vec::new();

    let mut remaining = text;
    let mut current_start = line_start_ms;

    while let Some(open) = remaining.find('<') {
        clean_text.push_str(&remaining[..open]);
        if let Some(close) = remaining[open..].find('>') {
            let tag = &remaining[open + 1..open + close];
            if let Some(time_ms) = parse_lrc_timestamp(tag) {
                if !clean_text.is_empty() {
                    runs.push(TimedRun {
                        text: clean_text.trim().to_string(),
                        start_ms: current_start,
                        end_ms: time_ms,
                    });
                }
                current_start = time_ms;
            }
            remaining = &remaining[open + close + 1..];
        } else {
            clean_text.push_str(&remaining[open..]);
            break;
        }
    }
    clean_text.push_str(remaining);

    (clean_text.trim().to_string(), runs)
}

/// Parses Apple Music TTML XML formatted lyrics.
pub fn parse_ttml(xml: &str) -> Option<ParsedLyrics> {
    let doc = Document::parse(xml).ok()?;
    let mut lines = Vec::new();
    let mut has_word_sync = false;

    for node in doc.descendants().filter(|n| n.has_tag_name("p")) {
        let begin_str = node.attribute("begin").or_else(|| node.attribute("itunes:begin"));
        let end_str = node.attribute("end").or_else(|| node.attribute("itunes:end"));

        let start_ms = begin_str.and_then(parse_ttml_time).unwrap_or(0);
        let end_ms = end_str.and_then(parse_ttml_time).unwrap_or(start_ms + 3000);

        let mut line_text = String::new();
        let mut runs = Vec::new();

        for span in node.children().filter(|n| n.has_tag_name("span")) {
            let span_begin = span.attribute("begin").and_then(parse_ttml_time).unwrap_or(start_ms);
            let span_end = span.attribute("end").and_then(parse_ttml_time).unwrap_or(end_ms);
            if let Some(text) = span.text() {
                line_text.push_str(text);
                runs.push(TimedRun {
                    text: text.to_string(),
                    start_ms: span_begin,
                    end_ms: span_end,
                });
                has_word_sync = true;
            }
        }

        if line_text.is_empty()
            && let Some(text) = node.text() {
                line_text = text.to_string();
            }

        if !line_text.trim().is_empty() {
            lines.push(LyricLine {
                text: line_text.trim().to_string(),
                start_ms,
                end_ms,
                runs,
                is_instrumental_gap: false,
            });
        }
    }

    if lines.is_empty() {
        return None;
    }

    lines.sort_by_key(|l| l.start_ms);

    Some(ParsedLyrics {
        lines,
        is_synced: true,
        has_word_sync,
        raw_ttml: Some(xml.to_string()),
    })
}

fn parse_ttml_time(s: &str) -> Option<u64> {
    if let Some(rest) = s.strip_suffix("s") {
        let secs: f64 = rest.parse().ok()?;
        return Some((secs * 1000.0) as u64);
    }
    if let Some(rest) = s.strip_suffix("ms") {
        return rest.parse::<u64>().ok();
    }
    parse_lrc_timestamp(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let lrc = "[00:12.50]Hello world\n[00:15.80]Second line";
        let parsed = parse_lrc(lrc);
        assert!(parsed.is_synced);
        assert_eq!(parsed.lines.len(), 2);
        assert_eq!(parsed.lines[0].text, "Hello world");
        assert_eq!(parsed.lines[0].start_ms, 12500);
        assert_eq!(parsed.lines[0].end_ms, 15800);
        assert_eq!(parsed.lines[1].text, "Second line");
        assert_eq!(parsed.lines[1].start_ms, 15800);
    }
}

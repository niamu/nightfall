use serde_derive::{Deserialize, Serialize};
use std::{path::Path, process::Command, str, time::Duration};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct FFPWrapper {
    ffpstream: Option<FFPStream>,
    corrupt: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FFPStream {
    chapters: Vec<Chapter>,
    streams: Vec<Stream>,
    format: Format,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: i64,
    pub time_base: String,
    pub start: i64,
    pub start_time: String,
    pub end: i64,
    pub end_time: String,
    pub tags: Option<Tags>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stream {
    pub index: i64,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: Option<String>,
    pub codec_type: String,
    pub codec_time_base: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub coded_width: Option<i64>,
    pub coded_height: Option<i64>,
    pub display_aspect_ratio: Option<String>,
    pub is_avc: Option<String>,
    pub tags: Option<Tags>,
    pub sample_rate: Option<String>,
    pub channels: Option<i64>,
    pub channel_layout: Option<String>,
    pub bit_rate: Option<String>,
    pub duration_ts: Option<i64>,
    pub duration: Option<String>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tags {
    pub language: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "BPS-eng")]
    pub bps_eng: Option<String>,
    #[serde(rename = "DURATION-eng")]
    pub duration_eng: Option<String>,
    #[serde(rename = "NUMBER_OF_FRAMES-eng")]
    pub number_of_frames_eng: Option<String>,
    #[serde(rename = "NUMBER_OF_BYTES-eng")]
    pub number_of_bytes_eng: Option<String>,
    #[serde(rename = "_STATISTICS_WRITING_APP-eng")]
    pub statistics_writing_app_eng: Option<String>,
    #[serde(rename = "_STATISTICS_WRITING_DATE_UTC-eng")]
    pub statistics_writing_date_utc_eng: Option<String>,
    #[serde(rename = "_STATISTICS_TAGS-eng")]
    pub statistics_tags_eng: Option<String>,
    pub filename: Option<String>,
    pub mimetype: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Format {
    pub filename: String,
    pub nb_streams: i64,
    pub nb_programs: i64,
    pub format_name: String,
    pub format_long_name: String,
    pub start_time: String,
    pub duration: String,
    pub size: String,
    pub bit_rate: String,
}

pub struct FFProbeCtx {
    ffprobe_bin: String,
}

fn format_timecode(nanos: i64) -> String {
    let d = Duration::from_nanos(nanos as u64);

    let total_secs = d.as_secs();

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = d.subsec_millis();

    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

impl FFProbeCtx {
    pub fn new(ffprobe_bin: &'static str) -> Self {
        Self {
            ffprobe_bin: ffprobe_bin.to_owned(),
        }
    }

    pub fn get_meta(&self, file: &Path) -> Result<FFPWrapper, std::io::Error> {
        let probe = Command::new(self.ffprobe_bin.clone())
            .arg(file.to_str().unwrap())
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_chapters")
            .arg("-show_streams")
            .arg("-show_format")
            .output()?;

        let json = String::from_utf8_lossy(probe.stdout.as_slice());

        let de: FFPWrapper = serde_json::from_str(&json).map_or_else(
            |_| FFPWrapper {
                ffpstream: None,
                corrupt: Some(true),
            },
            |x| FFPWrapper {
                ffpstream: Some(x),
                corrupt: None,
            },
        );

        Ok(de)
    }

    pub fn get_chapters_webvtt(&self, file: &Path) -> Result<String, std::io::Error> {
        let chapters = self
            .get_meta(&file)?
            .ffpstream
            .map(|s| s.chapters)
            .unwrap_or_default();

        let mut output = String::new();

        if chapters.len() == 0 {
            return Ok(output);
        };

        output.push_str("WEBVTT\n\n");

        for (i, chapter) in chapters.iter().enumerate() {
            let default_title = format!("Chapter {}", i + 1);

            let title = chapter
                .tags
                .as_ref()
                .and_then(|tags| tags.title.as_deref())
                .unwrap_or(&default_title);

            output.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                format_timecode(chapter.start),
                format_timecode(chapter.end),
                title,
            ));
        }

        Ok(output)
    }
}

use std::fmt;

use ffmpeg_next::{
    ChannelLayout, Packet, Rational, codec, decoder, ffi, frame, packet,
    util::format::{Pixel, Sample},
};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncReadExt, net::TcpStream};

const SC_PACKET_FLAG_SESSION: u64 = 1u64 << 63;
const SC_PACKET_FLAG_CONFIG: u64 = 1u64 << 62;
const SC_PACKET_FLAG_KEY_FRAME: u64 = 1u64 << 61;
const SC_PACKET_PTS_MASK: u64 = SC_PACKET_FLAG_KEY_FRAME - 1;
const MAX_MEDIA_PACKET_SIZE: usize = 64 * 1024 * 1024;
const SC_PACKET_TIME_BASE: Rational = Rational(1, 1_000_000);
const SC_AUDIO_SAMPLE_RATE: i32 = 48_000;

pub struct MediaPacket {
    data: Vec<u8>,
    pts: Option<i64>,
    is_config: bool,
    is_key_frame: bool,
    session: Option<MediaSession>,
}

#[derive(Debug, Clone, Copy)]
pub struct MediaSession {
    pub width: u32,
    pub height: u32,
    pub is_client_resize: bool,
}

impl MediaPacket {
    pub fn session(&self) -> Option<MediaSession> {
        self.session
    }

    pub fn is_config(&self) -> bool {
        self.is_config
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn ffmpeg_packet(data: Vec<u8>, pts: Option<i64>, is_key_frame: bool) -> Packet {
        let mut packet = Packet::copy(&data);
        packet.set_pts(pts);
        packet.set_dts(pts);

        if is_key_frame {
            packet.set_flags(packet.flags() | packet::Flags::KEY);
        }

        packet
    }

    pub fn into_ffmpeg_packet(self) -> Packet {
        Self::ffmpeg_packet(self.data, self.pts, self.is_key_frame)
    }
}

pub async fn read_media_packet(socket: &mut TcpStream) -> std::result::Result<MediaPacket, String> {
    // read header
    let mut header: [u8; 12] = [0; 12];
    socket
        .read_exact(&mut header)
        .await
        .map_err(|e| format!("{}: {}", t!("scrcpy.failedToReadFrameHeader"), e))?;

    let pts_flags = u64::from_be_bytes(header[0..8].try_into().unwrap());
    let len = u32::from_be_bytes(header[8..12].try_into().unwrap()) as usize;

    if (pts_flags & SC_PACKET_FLAG_SESSION) != 0 {
        return Ok(MediaPacket {
            data: Vec::new(),
            pts: None,
            is_config: false,
            is_key_frame: false,
            session: Some(MediaSession {
                width: pts_flags as u32,
                height: len as u32,
                is_client_resize: (pts_flags & (1u64 << 32)) != 0,
            }),
        });
    }

    if len > MAX_MEDIA_PACKET_SIZE {
        return Err(format!(
            "{}: packet too large ({len})",
            t!("scrcpy.failedToReadFrameHeader")
        ));
    }

    // read data
    let mut packet_data = vec![0u8; len];
    socket
        .read_exact(&mut packet_data)
        .await
        .map_err(|e| format!("{}: {}", t!("scrcpy.failedToReadFrameHeader"), e))?;

    let is_config = (pts_flags & SC_PACKET_FLAG_CONFIG) != 0;
    let pts = if is_config {
        None
    } else {
        Some((pts_flags & SC_PACKET_PTS_MASK) as i64)
    };

    Ok(MediaPacket {
        data: packet_data,
        pts,
        is_config,
        is_key_frame: (pts_flags & SC_PACKET_FLAG_KEY_FRAME) != 0,
        session: None,
    })
}

// Video Codec Constants
pub const SC_CODEC_ID_H264: u32 = 0x68_32_36_34;
pub const SC_CODEC_ID_H265: u32 = 0x68_32_36_35;
pub const SC_CODEC_ID_AV1: u32 = 0x00_61_76_31;
pub const SC_CODEC_ID_OPUS: u32 = 0x6f_70_75_73;
pub const SC_CODEC_ID_AAC: u32 = 0x00_61_61_63;
pub const SC_CODEC_ID_FLAC: u32 = 0x66_6c_61_63;
pub const SC_CODEC_ID_RAW: u32 = 0x00_72_61_77;

pub struct PacketMerger {
    config: Option<Vec<u8>>,
}

impl PacketMerger {
    pub fn new() -> Self {
        PacketMerger { config: None }
    }

    pub fn merge(&mut self, media_packet: MediaPacket) -> Option<Packet> {
        if media_packet.is_config {
            self.config = Some(media_packet.data);
            return None;
        }

        let Some(config_data) = self.config.take() else {
            return Some(media_packet.into_ffmpeg_packet());
        };

        let mut merged_data = Vec::with_capacity(config_data.len() + media_packet.data.len());
        merged_data.extend_from_slice(&config_data);
        merged_data.extend_from_slice(&media_packet.data);

        Some(MediaPacket::ffmpeg_packet(
            merged_data,
            media_packet.pts,
            media_packet.is_key_frame,
        ))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    H265,
    AV1,
}

impl From<VideoCodec> for codec::Id {
    fn from(codec: VideoCodec) -> Self {
        match codec {
            VideoCodec::H264 => Self::H264,
            VideoCodec::H265 => Self::HEVC,
            VideoCodec::AV1 => Self::AV1,
        }
    }
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VideoCodec::H264 => "h264",
            VideoCodec::H265 => "h265",
            VideoCodec::AV1 => "av1",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AudioCodec {
    Opus,
    Aac,
    Flac,
    Raw,
}

impl From<AudioCodec> for codec::Id {
    fn from(codec: AudioCodec) -> Self {
        match codec {
            AudioCodec::Opus => Self::OPUS,
            AudioCodec::Aac => Self::AAC,
            AudioCodec::Flac => Self::FLAC,
            AudioCodec::Raw => Self::PCM_S16LE,
        }
    }
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            AudioCodec::Opus => "opus",
            AudioCodec::Aac => "aac",
            AudioCodec::Flac => "flac",
            AudioCodec::Raw => "raw",
        };
        write!(f, "{}", s)
    }
}

pub struct AudioDecoder {
    pub decoder: decoder::Audio,
    pub codec_id: AudioCodec,
}

impl AudioDecoder {
    pub fn new(codec_id: AudioCodec, config: Option<&[u8]>) -> std::result::Result<Self, String> {
        let codec = decoder::find(codec_id.into())
            .ok_or_else(|| format!("FFmpeg decoder not found: {codec_id}"))?;
        let mut codec_context = codec::Context::new_with_codec(codec);
        configure_audio_decoder_context(&mut codec_context);
        if let Some(config) = config {
            set_decoder_extradata(&mut codec_context, config)?;
        }
        let mut decoder = codec_context.decoder();
        decoder.set_packet_time_base(SC_PACKET_TIME_BASE);
        let audio_decoder = decoder
            .audio()
            .map_err(|e| format!("Failed to open FFmpeg decoder: {e}"))?;

        Ok(Self {
            decoder: audio_decoder,
            codec_id,
        })
    }

    pub fn output_sample_format() -> Sample {
        Sample::F32(ffmpeg_next::format::sample::Type::Packed)
    }
}

fn configure_audio_decoder_context(codec_context: &mut codec::Context) {
    unsafe {
        let raw = codec_context.as_mut_ptr();
        (*raw).sample_rate = SC_AUDIO_SAMPLE_RATE;
        (*raw).ch_layout = ChannelLayout::STEREO.into();
    }
}

fn set_decoder_extradata(
    codec_context: &mut codec::Context,
    config: &[u8],
) -> std::result::Result<(), String> {
    if config.is_empty() {
        return Ok(());
    }

    let allocation_size = config.len() + ffi::AV_INPUT_BUFFER_PADDING_SIZE as usize;
    unsafe {
        let extradata = ffi::av_mallocz(allocation_size);
        if extradata.is_null() {
            return Err("Failed to allocate FFmpeg extradata".to_string());
        }

        std::ptr::copy_nonoverlapping(config.as_ptr(), extradata as *mut u8, config.len());
        let raw = codec_context.as_mut_ptr();
        (*raw).extradata = extradata as *mut u8;
        (*raw).extradata_size = config.len() as i32;
    }

    Ok(())
}

pub struct VideoDecoder {
    pub decoder: decoder::Video,
    pub codec_id: VideoCodec,
    pub width: u32,
    pub height: u32,
    pixel_format: Option<Pixel>,
    pub must_merge_config: bool,
    pub packet_merger: PacketMerger,
}

impl VideoDecoder {
    pub fn new(codec_id: VideoCodec, width: u32, height: u32) -> std::result::Result<Self, String> {
        let codec = decoder::find(codec_id.into())
            .ok_or_else(|| format!("FFmpeg decoder not found: {codec_id}"))?;
        let mut codec_context = codec::Context::new_with_codec(codec);
        let flags = unsafe {
            let raw_flags = (*codec_context.as_mut_ptr()).flags;
            let flags = codec::Flags::from_bits(raw_flags as std::ffi::c_uint)
                .unwrap_or(codec::Flags::empty());
            flags | codec::Flags::LOW_DELAY
        };
        codec_context.set_flags(flags);
        let video_decoder = codec_context
            .decoder()
            .video()
            .map_err(|e| format!("Failed to open FFmpeg decoder: {e}"))?;

        Ok(Self {
            decoder: video_decoder,
            codec_id,
            width,
            height,
            must_merge_config: matches!(codec_id, VideoCodec::H264 | VideoCodec::H265),
            packet_merger: PacketMerger::new(),
            pixel_format: None,
        })
    }

    pub fn update(&mut self, decoded: &frame::Video) -> std::result::Result<bool, String> {
        let width = decoded.width();
        let height = decoded.height();
        let format = decoded.format();

        if width != self.width || height != self.height || self.pixel_format != Some(format) {
            self.width = width;
            self.height = height;
            self.pixel_format = Some(format);

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YuvPlaneLayout {
    pub y_width: u32,
    pub y_height: u32,
    pub uv_width: u32,
    pub uv_height: u32,
}

impl YuvPlaneLayout {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            y_width: width,
            y_height: height,
            uv_width: width.div_ceil(2),
            uv_height: height.div_ceil(2),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YuvColorInfo {
    pub matrix: YuvMatrix,
    pub range: YuvRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YuvMatrix {
    Bt601,
    Bt709,
    Bt2020,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YuvRange {
    Limited,
    Full,
}

pub enum VideoMsg {
    Yuv420p {
        y: Vec<u8>,
        u: Vec<u8>,
        v: Vec<u8>,
        width: u32,
        height: u32,
        planes: YuvPlaneLayout,
        color: YuvColorInfo,
    },
    Nv12 {
        y: Vec<u8>,
        uv: Vec<u8>,
        width: u32,
        height: u32,
        planes: YuvPlaneLayout,
        color: YuvColorInfo,
    },
    Close,
}

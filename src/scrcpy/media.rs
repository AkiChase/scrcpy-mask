use std::{fmt, io::Write};

use bevy::ecs::error::Result;
use ffmpeg_next::{Packet, codec, decoder, format::Pixel, frame, packet, software::scaling};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncReadExt, net::TcpStream};

const SC_PACKET_FLAG_CONFIG: u64 = 1u64 << 63;
const SC_PACKET_FLAG_KEY_FRAME: u64 = 1u64 << 62;
const SC_PACKET_PTS_MASK: u64 = SC_PACKET_FLAG_KEY_FRAME - 1;
pub async fn read_media_packet(socket: &mut TcpStream) -> Result<Packet, String> {
    // read header
    let mut header: [u8; 12] = [0; 12];
    socket
        .read_exact(&mut header)
        .await
        .map_err(|e| format!("{}: {}", t!("scrcpy.failedToReadFrameHeader"), e))?;

    let pts_flags = u64::from_be_bytes(header[0..8].try_into().unwrap());
    let len = u32::from_be_bytes(header[8..12].try_into().unwrap()) as usize;

    // read data
    let mut packet_data = vec![0u8; len];
    socket
        .read_exact(&mut packet_data)
        .await
        .map_err(|e| format!("{}: {}", t!("scrcpy.failedToReadFrameHeader"), e))?;

    let mut packet = Packet::copy(&packet_data);
    if (pts_flags & SC_PACKET_FLAG_CONFIG) != 0 {
        packet.set_pts(None);
    } else {
        packet.set_pts(Some((pts_flags & SC_PACKET_PTS_MASK) as i64));
    }

    if (pts_flags & SC_PACKET_FLAG_KEY_FRAME) != 0 {
        packet.set_flags(packet.flags() | packet::Flags::KEY);
    }

    packet.set_dts(packet.pts());

    Ok(packet)
}

// Video Codec Constants
pub const SC_CODEC_ID_H264: u32 = 0x68_32_36_34;
pub const SC_CODEC_ID_H265: u32 = 0x68_32_36_35;
pub const SC_CODEC_ID_AV1: u32 = 0x00_61_76_31;

pub struct PacketMerger {
    config: Option<Vec<u8>>,
}

impl PacketMerger {
    pub fn new() -> Self {
        PacketMerger { config: None }
    }

    pub fn merge(&mut self, packet: &mut Packet) {
        let is_config = packet.pts().is_none();

        if is_config {
            if let Some(data) = packet.data() {
                self.config = Some(data.to_vec());
            } else {
                self.config = Some(Vec::new());
            }
        } else if let Some(config_data) = &self.config {
            let config_size = config_data.len();

            let original_data = if let Some(data) = packet.data() {
                data.to_vec()
            } else {
                Vec::new()
            };
            let media_size = original_data.len();
            let new_size = config_size + media_size;

            let mut new_data = Vec::with_capacity(new_size);
            new_data.extend_from_slice(config_data);
            new_data.extend_from_slice(&original_data);

            packet.grow(new_size);
            packet.data_mut().unwrap().write_all(&new_data).unwrap();

            self.config = None;
        }
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

pub struct VideoDecoder {
    pub decoder: decoder::Video,
    pub scaler: Option<scaling::Context>,
    pub width: u32,
    pub height: u32,
    pub frame_size: usize,
    pub must_merge_config: bool,
    pub packet_merger: PacketMerger,
}

impl VideoDecoder {
    pub fn new(codec_id: VideoCodec, width: u32, height: u32) -> Self {
        let codec = decoder::find(codec_id.into()).unwrap();
        let mut codec_context = codec::Context::new_with_codec(codec);
        let flags = unsafe {
            let raw_flags = (*codec_context.as_mut_ptr()).flags;
            let flags = codec::Flags::from_bits(raw_flags as std::ffi::c_uint)
                .unwrap_or(codec::Flags::empty());
            flags | codec::Flags::LOW_DELAY
        };
        codec_context.set_flags(flags);
        let video_decoder = codec_context.decoder().video().unwrap();

        Self {
            decoder: video_decoder,
            scaler: None,
            width,
            height,
            must_merge_config: matches!(codec_id, VideoCodec::H264 | VideoCodec::H265),
            packet_merger: PacketMerger::new(),
            frame_size: (width * height * 4) as usize,
        }
    }

    pub fn update(&mut self) -> bool {
        let width = self.decoder.width();
        let height = self.decoder.height();
        if self.scaler.is_none() || width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.scaler = Some(
                scaling::Context::get(
                    self.decoder.format(),
                    width,
                    height,
                    Pixel::RGBA,
                    width,
                    height,
                    scaling::Flags::BILINEAR,
                )
                .unwrap(),
            );
            self.frame_size = (width * height * 4) as usize;

            true
        } else {
            false
        }
    }

    pub fn conver_to_rgba(&mut self, decoded: &frame::Video) -> frame::Video {
        let mut rgba_frame = frame::Video::empty();
        self.scaler
            .as_mut()
            .unwrap()
            .run(decoded, &mut rgba_frame)
            .unwrap();
        rgba_frame
    }
}

pub enum VideoMsg {
    Data {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Close,
}

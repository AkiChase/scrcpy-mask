use std::time::Duration;

use ffmpeg_next::{
    Error as FfmpegError, error,
    format::Pixel,
    frame,
    util::color::{Range, Space},
};
use rust_i18n::t;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::UnboundedSender,
        oneshot, watch,
    },
    time::timeout,
};
use tokio_util::sync::CancellationToken;

use crate::{
    mask::mask_command::MaskCommand,
    scrcpy::{
        control_msg::{ScrcpyControlMsg, ScrcpyDeviceMsg},
        media::{
            SC_CODEC_ID_AV1, SC_CODEC_ID_H264, SC_CODEC_ID_H265, VideoCodec, VideoDecoder,
            VideoMsg, YuvColorInfo, YuvMatrix, YuvPlaneLayout, YuvRange, read_media_packet,
        },
    },
    utils::{LatestVideoFrame, share::ControlledDevice},
};

pub struct ScrcpyConnection {
    pub socket: TcpStream,
}

impl ScrcpyConnection {
    pub fn new(socket: TcpStream) -> Self {
        ScrcpyConnection { socket }
    }

    async fn read_device_metadata(&mut self, scid: String) -> Result<(), String> {
        // read metadata (device name)
        let mut buf: [u8; 64] = [0; 64];
        match self.socket.read(&mut buf).await {
            Err(e) => Err(format!(
                "{}: {}",
                t!("scrcpy.failedToReadControlMetadata"),
                e
            )),
            Ok(0) => Err(format!(
                "{}: None",
                t!("scrcpy.failedToReadControlMetadata")
            )),
            Ok(n) => {
                let mut end = n;
                while buf[end - 1] == 0 {
                    end -= 1;
                }
                // update device name
                if let Ok(device_name_raw) = std::str::from_utf8(&buf[..n]) {
                    let device_name = device_name_raw.trim_end_matches(char::from(0));
                    ControlledDevice::update_device_name(scid, device_name.to_string()).await;
                } else {
                    log::warn!("[Controller] {}", t!("scrcpy.invalidDeviceName"));
                    ControlledDevice::update_device_name(scid, "INVALID_NAME".to_string()).await;
                }
                Ok(())
            }
        }
    }

    async fn control_writer(
        mut write_half: OwnedWriteHalf,
        token: CancellationToken,
        mut cs_rx: broadcast::Receiver<ScrcpyControlMsg>,
        mut watch_rx: watch::Receiver<(u32, u32)>,
    ) {
        tokio::select! {
            _ = token.cancelled()=>{
                log::info!("[Controller] {}", t!("scrcpy.controlConnectionCancelled"));
            }
            _ = async {
                loop {
                    match cs_rx.recv().await {
                        Ok(mut msg) => {
                                // scale position
                                match &mut msg {
                                    ScrcpyControlMsg::InjectTouchEvent {
                                        x,
                                        y,
                                        w,
                                        h,
                                        action: _,
                                        pointer_id: _,
                                        pressure: _,
                                        action_button: _,
                                        buttons: _,
                                    } => {
                                        let (device_w, device_h) = watch_rx.borrow_and_update().clone();
                                        let (old_x, old_y) = (*x, *y);
                                        let (old_w, old_h) = (*w, *h);
                                        *x = old_x * device_w as i32 / old_w as i32;
                                        *y = old_y * device_h as i32 / old_h as i32;
                                        *w = device_w as u16;
                                        *h = device_h as u16;
                                    }
                                    ScrcpyControlMsg::InjectScrollEvent {
                                        x,
                                        y,
                                        w,
                                        h,
                                        hscroll: _,
                                        vscroll: _,
                                        buttons: _,
                                    } => {
                                        let (device_w, device_h) = watch_rx.borrow_and_update().clone();
                                        let (old_x, old_y) = (*x, *y);
                                        let (old_w, old_h) = (*w, *h);
                                        *x = old_x * device_w as i32 / old_w as i32;
                                        *y = old_y * device_h as i32 / old_h as i32;
                                        *w = device_w as u16;
                                        *h = device_h as u16;
                                    }
                                    _ => {}
                                };
                                let data:Vec<u8> = msg.into();
                                if let Err(e) = write_half.write_all(&data).await {
                                    log::error!("[Controller] {}: {}", t!("scrcpy.controlConnWriteFailed"),e);
                                }
                        }
                        Err(RecvError::Lagged(skipped)) => {
                            log::warn!("[Controller] {}",t!("controller.csReceiverLagged", skipped => skipped));
                        }
                        Err(e) => {
                            log::info!("[Controller] {}: {}", t!("scrcpy.controlChannelClosed"),e);
                            break;
                        }
                    }
                }
            }=>{
                log::error!("[Controller] {}", t!("scrcpy.controlCnnShutdownUnexpectedly"));
            }
        }
        timeout(Duration::from_millis(500), write_half.shutdown())
            .await
            .ok();
    }

    async fn control_reader_handler(
        mut read_half: OwnedReadHalf,
        cr_tx: UnboundedSender<ScrcpyDeviceMsg>,
        watch_tx: watch::Sender<(u32, u32)>,
        scid: &str,
        main: bool,
    ) {
        loop {
            match ScrcpyDeviceMsg::read_msg(&mut read_half, scid.to_string()).await {
                Ok(msg) => {
                    if let ScrcpyDeviceMsg::Rotation {
                        rotation: _,
                        width,
                        height,
                        scid,
                    } = msg.clone()
                    {
                        ControlledDevice::update_device_size(scid, (width, height)).await;
                        watch_tx.send((width, height)).unwrap();
                    }
                    // only forward other message from main device
                    if main {
                        cr_tx.send(msg).unwrap();
                    }
                }
                Err(e) => {
                    log::error!("[Controller] {}", e);
                    break;
                }
            };
        }
    }

    async fn control_reader(
        read_half: OwnedReadHalf,
        token: CancellationToken,
        cr_tx: UnboundedSender<ScrcpyDeviceMsg>,
        watch_tx: watch::Sender<(u32, u32)>,
        scid: &str,
        main: bool,
    ) {
        tokio::select! {
            _ = token.cancelled()=>{
                log::info!("[Controller] {}", t!("scrcpy.controlConnectionReaderCancelled"));
            }
            _ = Self::control_reader_handler(read_half, cr_tx, watch_tx, scid, main)=>{
                log::error!("[Controller] {}", t!("scrcpy.controlReadShutdownUnexpectedly"));
            }
        }
        // no need to shutdown the read_half
    }

    pub async fn handle_control(
        mut self,
        cs_rx: broadcast::Receiver<ScrcpyControlMsg>,
        cr_tx: UnboundedSender<ScrcpyDeviceMsg>,
        m_tx: crossbeam_channel::Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
        scid: String,
        main: bool,
        token: CancellationToken,
        meta_flag: bool,
    ) {
        log::info!("[Controller] {}", t!("scrcpy.handleControlConnection"));
        if meta_flag {
            if let Err(e) = self.read_device_metadata(scid.to_string()).await {
                log::error!("[Controller] {}", e);
                token.cancel();
                return;
            }
        }

        let (read_half, write_half) = self.socket.into_split();
        let finnal_token = token.clone();
        let token_copy = token.clone();
        let (watch_tx, watch_rx) = watch::channel::<(u32, u32)>((0, 0)); // share device size with writer
        if main {
            let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
            m_tx.send((
                MaskCommand::DeviceConnectionChange { connect: true },
                oneshot_tx,
            ))
            .unwrap();
            oneshot_rx.await.unwrap().unwrap();
        }

        tokio::select! {
            _ = Self::control_writer(write_half, token, cs_rx, watch_rx) => {finnal_token.cancel();}
            _ = Self::control_reader(read_half, token_copy, cr_tx, watch_tx, &scid, main) => {finnal_token.cancel();}
        }

        log::info!("[Controller] {}", t!("scrcpy.controlConnectionClosed"));
        if main {
            let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
            m_tx.send((
                MaskCommand::DeviceConnectionChange { connect: false },
                oneshot_tx,
            ))
            .unwrap();
            oneshot_rx.await.unwrap().unwrap();
        }
    }

    async fn video_handler(&mut self, v_tx: LatestVideoFrame) {
        // read metadata
        let mut buf: [u8; 12] = [0; 12];
        let mut video_decoder = match self.socket.read_exact(&mut buf).await {
            Err(_) => {
                log::error!("[Controller] {}", t!("scrcpy.failedToReadVideoMetadata"));
                return;
            }
            Ok(_) => {
                let raw_codec_id = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let width = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
                let height = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);

                let codec_id = match raw_codec_id {
                    SC_CODEC_ID_H264 => {
                        log::info!("[Controller] {}: H264", t!("scrcpy.videoCodec"));
                        VideoCodec::H264
                    }
                    SC_CODEC_ID_H265 => {
                        log::info!("[Controller] {}: H265", t!("scrcpy.videoCodec"));
                        VideoCodec::H265
                    }
                    SC_CODEC_ID_AV1 => {
                        log::info!("[Controller] {}: AV1", t!("scrcpy.videoCodec"));
                        VideoCodec::AV1
                    }
                    _ => {
                        log::error!(
                            "[Controller] {}: 0x{:x}",
                            t!("scrcpy.invalidVideoCodec"),
                            raw_codec_id
                        );
                        return;
                    }
                };
                match VideoDecoder::new(codec_id, width, height) {
                    Ok(video_decoder) => video_decoder,
                    Err(e) => {
                        log::error!("[Controller] {}", e);
                        return;
                    }
                }
            }
        };

        // read video packets
        loop {
            match read_media_packet(&mut self.socket).await {
                Ok(media_packet) => {
                    let packet = if video_decoder.must_merge_config {
                        video_decoder.packet_merger.merge(media_packet)
                    } else if media_packet.is_config() {
                        None
                    } else {
                        Some(media_packet.into_ffmpeg_packet())
                    };

                    let Some(packet) = packet else {
                        continue;
                    };

                    match video_decoder.decoder.send_packet(&packet) {
                        Ok(()) => {}
                        Err(e) if is_ffmpeg_again(e) => {
                            if !drain_video_decoder(&mut video_decoder, &v_tx) {
                                break;
                            }
                            if let Err(e) = video_decoder.decoder.send_packet(&packet) {
                                log::warn!("[Controller] Failed to send video packet: {}", e);
                                continue;
                            }
                        }
                        Err(e) => {
                            log::warn!("[Controller] Failed to send video packet: {}", e);
                            continue;
                        }
                    }

                    if !drain_video_decoder(&mut video_decoder, &v_tx) {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("[Controller] {}", e);
                    break;
                }
            }
        }
    }

    pub async fn handle_video(
        mut self,
        token: CancellationToken,
        v_tx: LatestVideoFrame,
        meta_flag: bool,
        scid: &str,
    ) {
        log::info!("[Controller] {}", t!("scrcpy.handleVideoConnection"));
        if meta_flag {
            if let Err(e) = self.read_device_metadata(scid.to_string()).await {
                log::error!("[Controller] {}", e);
                token.cancel();
                return;
            }
        }

        let finnal_token = token.clone();

        tokio::select! {
            _ = token.cancelled()=>{
                log::info!("[Controller] {}", t!("scrcpy.videoConnectionReaderCancelled"));
            }
            _ = self.video_handler(v_tx.clone())=>{
                log::error!("[Controller] {}", t!("scrcpy.videoReadShutdownUnexpectedly"));
                finnal_token.cancel();
            }
        }
        v_tx.send(VideoMsg::Close);
        log::info!("[Controller] {}", t!("scrcpy.videoConnectionClosed"));
        self.socket.shutdown().await.unwrap();
    }
}

fn drain_video_decoder(
    video_decoder: &mut VideoDecoder,
    v_tx: &LatestVideoFrame,
) -> bool {
    loop {
        let mut decoded = frame::Video::empty();
        match video_decoder.decoder.receive_frame(&mut decoded) {
            Ok(()) => {
                let format_changed = match video_decoder.update(&decoded) {
                    Ok(changed) => changed,
                    Err(e) => {
                        log::warn!("[Controller] {}", e);
                        continue;
                    }
                };

                if format_changed {
                    log_video_frame_metadata(video_decoder, &decoded);
                }

                let color = map_yuv_color_info(&decoded, format_changed);
                let planes = YuvPlaneLayout::new(video_decoder.width, video_decoder.height);

                match decoded.format() {
                    Pixel::YUV420P => {
                        let y_size = (planes.y_width * planes.y_height) as usize;
                        let uv_size = (planes.uv_width * planes.uv_height) as usize;
                        let mut y = v_tx.take_buffer(y_size);
                        let mut u = v_tx.take_buffer(uv_size);
                        let mut v = v_tx.take_buffer(uv_size);

                        copy_plane(
                            &decoded,
                            0,
                            planes.y_width as usize,
                            planes.y_height as usize,
                            &mut y,
                        );
                        copy_plane(
                            &decoded,
                            1,
                            planes.uv_width as usize,
                            planes.uv_height as usize,
                            &mut u,
                        );
                        copy_plane(
                            &decoded,
                            2,
                            planes.uv_width as usize,
                            planes.uv_height as usize,
                            &mut v,
                        );

                        v_tx.send(VideoMsg::Yuv420p {
                            y,
                            u,
                            v,
                            width: video_decoder.width,
                            height: video_decoder.height,
                            planes,
                            color,
                        });
                    }
                    Pixel::NV12 => {
                        let y_size = (planes.y_width * planes.y_height) as usize;
                        let uv_size = (planes.uv_width * planes.uv_height * 2) as usize;
                        let mut y = v_tx.take_buffer(y_size);
                        let mut uv = v_tx.take_buffer(uv_size);

                        copy_plane(
                            &decoded,
                            0,
                            planes.y_width as usize,
                            planes.y_height as usize,
                            &mut y,
                        );
                        copy_plane(
                            &decoded,
                            1,
                            planes.uv_width as usize * 2,
                            planes.uv_height as usize,
                            &mut uv,
                        );

                        v_tx.send(VideoMsg::Nv12 {
                            y,
                            uv,
                            width: video_decoder.width,
                            height: video_decoder.height,
                            planes,
                            color,
                        });
                    }
                    format => {
                        log::error!(
                            "[Controller] Unsupported video pixel format: codec={}, format={format:?}, width={}, height={}, color_space={:?}, color_range={:?}, primaries={:?}, transfer={:?}, chroma_location={:?}",
                            video_decoder.codec_id,
                            decoded.width(),
                            decoded.height(),
                            decoded.color_space(),
                            decoded.color_range(),
                            decoded.color_primaries(),
                            decoded.color_transfer_characteristic(),
                            decoded.chroma_location(),
                        );
                        v_tx.send(VideoMsg::Close);
                        return false;
                    }
                }
            }
            Err(e) if is_ffmpeg_again(e) => break,
            Err(FfmpegError::Eof) => break,
            Err(e) => {
                log::warn!("[Controller] Failed to receive video frame: {}", e);
                break;
            }
        }
    }
    true
}



fn log_video_frame_metadata(video_decoder: &VideoDecoder, decoded: &frame::Video) {
    log::info!(
        "[Controller] Video frame format: codec={}, format={:?}, size={}x{}, color_space={:?}, color_range={:?}, primaries={:?}, transfer={:?}, chroma_location={:?}",
        video_decoder.codec_id,
        decoded.format(),
        decoded.width(),
        decoded.height(),
        decoded.color_space(),
        decoded.color_range(),
        decoded.color_primaries(),
        decoded.color_transfer_characteristic(),
        decoded.chroma_location(),
    );
}

fn map_yuv_color_info(decoded: &frame::Video, warn_on_assumption: bool) -> YuvColorInfo {
    let matrix = match decoded.color_space() {
        Space::BT470BG | Space::SMPTE170M => YuvMatrix::Bt601,
        Space::BT2020NCL | Space::BT2020CL => YuvMatrix::Bt2020,
        Space::BT709 => YuvMatrix::Bt709,
        other => {
            if warn_on_assumption {
                log::warn!(
                    "[Controller] Unknown video color matrix {:?}; assuming BT.709 for YUV shader",
                    other
                );
            }
            YuvMatrix::Bt709
        }
    };

    let range = match decoded.color_range() {
        Range::MPEG => YuvRange::Limited,
        Range::JPEG => YuvRange::Full,
        other => {
            if warn_on_assumption {
                log::warn!(
                    "[Controller] Unknown video color range {:?}; assuming limited range for YUV shader",
                    other
                );
            }
            YuvRange::Limited
        }
    };

    YuvColorInfo { matrix, range }
}

fn is_ffmpeg_again(error: FfmpegError) -> bool {
    matches!(error, FfmpegError::Other { errno } if errno == error::EAGAIN)
}

fn copy_plane(
    frame: &frame::Video,
    plane: usize,
    width_bytes: usize,
    height: usize,
    dst: &mut [u8],
) {
    let frame_size = width_bytes * height;
    let src = frame.data(plane);
    let src_stride = frame.stride(plane);

    if src_stride == width_bytes {
        dst[..frame_size].copy_from_slice(&src[..frame_size]);
        return;
    }

    for row in 0..height {
        let src_start = row * src_stride;
        let dst_start = row * width_bytes;
        dst[dst_start..dst_start + width_bytes]
            .copy_from_slice(&src[src_start..src_start + width_bytes]);
    }
}

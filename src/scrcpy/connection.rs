use std::time::Duration;

use ffmpeg_next::frame;
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
            VideoMsg, read_media_packet,
        },
    },
    utils::share::ControlledDevice,
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

    async fn video_handler(&mut self, v_tx: crossbeam_channel::Sender<VideoMsg>) {
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
                let video_decoder = VideoDecoder::new(codec_id, width, height);
                video_decoder
            }
        };

        // read video packets
        loop {
            match read_media_packet(&mut self.socket).await {
                Ok(mut packet) => {
                    if video_decoder.must_merge_config {
                        // merge config packet if needed
                        video_decoder.packet_merger.merge(&mut packet);
                    }

                    // no send config packet
                    if packet.pts().is_some() {
                        let decoded = {
                            let mut decoded = frame::Video::empty();
                            video_decoder.decoder.send_packet(&mut packet).unwrap();
                            video_decoder.decoder.receive_frame(&mut decoded).unwrap();
                            decoded
                        };
                        // update size after decoding video packet
                        video_decoder.update();

                        let rgb_frame = video_decoder.conver_to_rgba(&decoded);
                        let mut buf = Vec::with_capacity(video_decoder.frame_size);
                        buf.resize(video_decoder.frame_size, 0);
                        buf.copy_from_slice(rgb_frame.data(0));

                        v_tx.send(VideoMsg::Data {
                            data: buf,
                            width: video_decoder.width,
                            height: video_decoder.height,
                        })
                        .unwrap();
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
        v_tx: crossbeam_channel::Sender<VideoMsg>,
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
        v_tx.send(VideoMsg::Close).unwrap();
        log::info!("[Controller] {}", t!("scrcpy.videoConnectionClosed"));
        self.socket.shutdown().await.unwrap();
    }
}

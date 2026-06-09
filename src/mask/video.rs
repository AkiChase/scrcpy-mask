use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::scrcpy::media::VideoMsg;
use crate::utils::ChannelReceiverV;

#[derive(Resource, Default)]
pub struct VideoAttributes {
    width: u32,
    height: u32,
    image_handle: Option<Handle<Image>>,
}

impl VideoAttributes {
    fn update_image_asset(
        &mut self,
        width: u32,
        height: u32,
        images: &mut ResMut<Assets<Image>>,
        video_node: &mut Single<(&mut ImageNode, &mut VideoPlayer)>,
    ) -> &Handle<Image> {
        if self.image_handle.is_none() || self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            let mut image = Image::new_fill(
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[0, 0, 0, 0],
                TextureFormat::Bgra8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            );
            image.texture_descriptor.usage =
                TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
            let image_handle = images.add(image);
            video_node.0.image = image_handle.clone();
            self.image_handle = Some(image_handle);
        }
        self.image_handle.as_ref().unwrap()
    }
}

#[derive(Component)]
pub struct VideoPlayer;

pub fn handle_video_msg(
    v_rx: Res<ChannelReceiverV>,
    mut images: ResMut<Assets<Image>>,
    mut video_attr: Local<VideoAttributes>,
    mut video_node: Single<(&mut ImageNode, &mut VideoPlayer)>,
) {
    if let Some(msg) = v_rx.0.take() {
        match msg {
            VideoMsg::Data {
                data,
                width,
                height,
            } => {
                let image_handle =
                    video_attr.update_image_asset(width, height, &mut images, &mut video_node);
                if let Some(mut image) = images.get_mut(image_handle) {
                    if let Some(old_data) = image.data.replace(data) {
                        v_rx.0.recycle_buffer(old_data);
                    }
                }
            }
            VideoMsg::Close => {
                if let Some(image_handle) = video_attr.image_handle.take() {
                    if let Some(mut image) = images.get_mut(&image_handle) {
                        if let Some(old_data) = image.data.take() {
                            let length = old_data.len();
                            v_rx.0.recycle_buffer(old_data);
                            let mut clear_data = v_rx.0.take_buffer(length);
                            clear_data.fill(0);
                            image.data = Some(clear_data);
                        }
                    }
                    video_attr.image_handle = None;
                }
            }
        }
    }
}

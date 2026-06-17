use roseate_core::image_info::info::ImageInfo;

use crate::image_handler::image_resource::ImageResource;

pub struct LoadedImage {
    pub resource: ImageResource,
    pub image_info: ImageInfo,

    pub image_hash: u64,
}
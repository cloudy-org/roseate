use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}, path::Path, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use cirrus_egui::v1::{notifier::Notifier, scheduler::Scheduler};
use eframe::egui::Context;
use egui::{TextureFilter, TextureHandle, TextureOptions, TextureWrapMode};
use rfd::FileDialog;
use log::{debug, info, warn};
use monitor_downsampling::get_monitor_downsampling_size;
use optimization::ImageOptimizations;

use crate::{error::{Error, Result}, image::{backends::ImageProcessingBackend, image::{Image, ImageSizeT}, image_data::{ImageColourType, ImageData}, image_formats::ImageFormat, modifications::ImageModifications}, monitor_size::MonitorSize};

mod dynamic_sampling;

pub mod optimization;
pub mod monitor_downsampling;

// NOTE: need a better name for this
#[derive(Clone)]
pub enum ImageHandlerData {
    Texture(TextureHandle),
    EguiImage(egui::Image<'static>)
}

/// Struct that handles all the image loading logic in a thread safe 
/// manner to allow features such as background image loading / lazy loading.
/// 
/// ImageHandler struct is more ui facing, Image struct is lower-level.
pub struct ImageHandler {
    pub image: Option<Image>,
    pub image_optimizations: HashSet<ImageOptimizations>,
    pub data: Option<ImageHandlerData>,

    image_loading: bool,

    dynamic_sample_schedule: Option<Scheduler>,
    last_zoom_factor: f32,
    dynamic_sampling_new_resolution: ImageSizeT,
    dynamic_sampling_old_resolution: ImageSizeT,
    accumulated_zoom_factor_change: f32,
    monitor_downsampling_required: bool,
    load_image_texture: Arc<Mutex<bool>>,
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            image: None,
            image_optimizations: HashSet::new(),
            data: None,

            image_loading: false,

            dynamic_sample_schedule: None,
            last_zoom_factor: 1.0,
            dynamic_sampling_new_resolution: (0, 0),
            dynamic_sampling_old_resolution: (0, 0),
            accumulated_zoom_factor_change: 0.0,
            monitor_downsampling_required: false,
            load_image_texture: Arc::new(Mutex::new(false)),
        }
    }

    pub fn init_image(&mut self, image_path: &Path, image_optimizations: Vec<ImageOptimizations>) -> Result<()> {
        let image = Image::from_path(image_path)?;

        self.image = Some(image);
        self.image_optimizations = HashSet::from_iter(image_optimizations);

        Ok(())
    }

    pub fn select_image(&mut self, image_optimizations: Vec<ImageOptimizations>) -> Result<()> {
        let image_path = FileDialog::new()
            .add_filter("images", &["png", "jpeg", "jpg", "webp", "gif", "svg"])
            .pick_file();

        match image_path {
            Some(path) => {
                if !path.exists() {
                    return Err(
                        Error::FileNotFound(
                            None,
                            path,
                            "The file picked in the file selector does not exist!".to_string()
                        )
                    )
                }

                self.init_image(&path, image_optimizations)?;

                Ok(())
            },
            None => Err(
                Error::NoFileSelected(None)
            )
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        zoom_factor: &f32,
        is_panning: bool,
        monitor_size: &MonitorSize,
        notifier: &mut Notifier,
        backend: ImageProcessingBackend
    ) {
        // I use an update function to keep the public 
        // fields update to date with their Arc<Mutex<T>> twins
        // and also now to perform dynamic downsampling.

        // if let Ok(value) = self.image_loaded_arc.try_lock() {
        //     self.image_loaded = value.clone(); // cloning here shouldn't be too expensive
        //     self.image_loading = false; // set that bitch back to false yeah
        // }

        self.load_texture_update(ctx);
        self.dynamic_sampling_update(zoom_factor, monitor_size);

        if let Some(schedule) = &mut self.dynamic_sample_schedule {
            // TODO: if we are still panning once we have stopped 
            // defer some addition seconds to the dynamic_sample_schedule.
            if !is_panning {
                if schedule.update().is_some() {
                    if self.dynamic_sampling_new_resolution == self.dynamic_sampling_old_resolution {
                        debug!(
                            "Will not schedule this dynamic sample ({:?} -> {:?}) \
                            as it's going to sample to the same resolution!",
                            self.dynamic_sampling_old_resolution,
                            self.dynamic_sampling_new_resolution
                        );
                        return;
                    }

                    self.load_image(
                        true,
                        notifier,
                        monitor_size,
                        backend
                    );
                }
            }
        }
    }

    fn load_texture_update(&mut self, ctx: &Context) {
        let reload_texture = match self.load_image_texture.try_lock() {
            Ok(load_image_texture_mutex) => *load_image_texture_mutex,
            Err(_) => false,
        };

        if reload_texture == false {
            return;
        }

        if let Some(image) = &self.image {
            if let Some(image_data) = image.image_data.lock().unwrap().as_ref() {
                let texture_options = TextureOptions {
                    magnification: TextureFilter::Linear,
                    minification: TextureFilter::Linear,
                    wrap_mode: TextureWrapMode::ClampToEdge,
                    mipmap_mode: None,
                };

                match image_data {
                    ImageData::Pixels(
                        (pixels, (width, height), image_colour_type)
                    ) => {
                        let image_size = [*width as usize, *height as usize];

                        debug!("Handing image texture to egui's backend to upload to the GPU...");

                        let texture = ctx.load_texture(
                            "image",
                            match image_colour_type {
                                ImageColourType::Grey | ImageColourType::GreyAlpha => {
                                    debug!("Rendering image as grey scale egui texture...");
                                    egui::ColorImage::from_gray(
                                        image_size, pixels
                                    )
                                },
                                ImageColourType::RGB => {
                                    debug!("Rendering image as rgb egui texture...");
                                    egui::ColorImage::from_rgb(
                                        image_size, pixels
                                    )
                                },
                                ImageColourType::RGBA => {
                                    debug!("Rendering image as rgba egui texture...");
                                    egui::ColorImage::from_rgba_unmultiplied(
                                        image_size, pixels
                                    )
                                },
                            },
                            texture_options
                        );

                        // Texture handle doesn't need forgetting like egui::Image 
                        // as it's smart enough to free itself from memory

                        ctx.forget_all_images(); // we want to free the rose image in 
                        // image selection menu and all other images from memory.

                        self.data = Some(ImageHandlerData::Texture(texture));
                    },
                    ImageData::StaticBytes(bytes ) => {
                        // load from bytes using egui's image loading logic.
                        let egui_image = egui::Image::from_bytes(
                            format!("bytes://image.{:#}", image.image_format),
                            egui::load::Bytes::Shared(bytes.clone()) // we can clone here 
                            // without turning into a java application as we're using arc
                        );

                        ctx.forget_all_images();

                        // forget last image if there was one
                        // if let Some(data) = self.data.as_ref() {
                        //     if let ImageHandlerData::EguiImage(image) = data {
                        //         if let Some(uri) = image.uri() {
                        //             println!("test!");
                        //             ctx.forget_image(uri);
                        //         }
                        //     }
                        // }

                        self.data = Some(
                            ImageHandlerData::EguiImage(
                                egui_image.texture_options(texture_options)
                            )
                        );
                    },
                };
            }
        };

        *self.load_image_texture.lock().unwrap() = false;
        self.image_loading = false;
    }

    /// Handles loading the image in a background thread or on the main thread. 
    /// Set `lazy_load` to `true` if you want the image to be loaded in the background on a separate thread.
    /// 
    /// Setting `lazy_load` to `false` **will block the main thread** until the image is loaded.
    pub fn load_image(&mut self, lazy_load: bool, notifier: &mut Notifier, monitor_size: &MonitorSize, backend: ImageProcessingBackend) {
        if self.image_loading {
            warn!("Not loading image as one is already being loaded!");
            return;
        }

        self.image_loading = true;

        notifier.set_loading(
            Some("Preparing to load image...".into())
        );

        let mut image = self.image.clone().expect(
            "You must run 'ImageHandler.init_image()' before using 'ImageHandler.load_image()'!"
        );

        notifier.set_loading(
            Some("Gathering necessary image modifications...".into())
        );

        let image_modifications = self.get_image_modifications(
            &monitor_size
        );

        let image_modifications_display = format!("{:?}", image_modifications);

        // Our svg implementation is very experimental. 
        // Also broken! https://github.com/cloudy-org/roseate/issues/66 
        // Let's warn the user.
        if ImageFormat::Svg == image.image_format {
            notifier.toast(
                "SVG files are experimental and broken! \
                Expect many bugs, inconstancies and performance / memory issues.",
                egui_notify::ToastLevel::Warning,
                |toast| {
                    toast.duration(Some(Duration::from_secs(8)));
                }
            );
        }

        // let image_loaded_arc = self.image_loaded_arc.clone();
        let mut notifier_clone = notifier.clone();
        let monitor_size_clone = monitor_size.clone();
        let load_image_texture_clone = self.load_image_texture.clone();

        let image_loaded = self.data.is_some();

        let loading_logic = move || {
            let now = Instant::now();
            let mut hasher = DefaultHasher::new();

            let result = match image_loaded {
                true => {
                    notifier_clone.set_loading(Some("Reloading image...".into()));

                    let result = image.reload_image(
                        &mut notifier_clone,
                        image_modifications,
                        &backend
                    );

                    debug!(
                        "Image reloaded in '{}' seconds using '{}' backend.",
                        now.elapsed().as_secs_f32(),
                        backend
                    );

                    result
                },
                false => {
                    notifier_clone.set_loading(Some("Loading image...".into()));

                    let result = image.load_image(
                        &mut notifier_clone,
                        &monitor_size_clone,
                        image_modifications,
                        &backend
                    );

                    info!(
                        "Image loaded in '{}' seconds using '{}' backend.", 
                        now.elapsed().as_secs_f32(), backend
                    );

                    result
                }
            };

            match result {
                Ok(()) => {
                    *load_image_texture_clone.lock().unwrap() = true;

                    image.hash(&mut hasher);

                    debug!("Image data hash: {}", hasher.finish());
                    debug!("Image current modifications: {}", image_modifications_display);
                },
                Err(error) => {
                    notifier_clone.toast(
                        Box::new(error),
                        egui_notify::ToastLevel::Error,
                        |toast| {
                            toast.duration(Some(Duration::from_secs(10)));
                        }
                    );
                },
            }

            notifier_clone.unset_loading();
        };

        if lazy_load {
            debug!("Lazy loading image (in a thread)...");
            thread::spawn(loading_logic);
        } else {
            debug!("Loading image in main thread...");
            loading_logic();
        }
    }

    // NOTE: this is temporary, it will be rewritten.
    // pub fn get_egui_image(&'_ mut self, ctx: &egui::Context) -> egui::Image<'_> {
    //     // assert!(
    //     //     self.image_loaded,
    //     //     "'ImageHandler::get_egui_image()' should never be called if 'self.image_loaded' is false!"
    //     // );

    //     let image = self.image.as_ref().unwrap();

    //     let mut hasher = DefaultHasher::new();
    //     image.hash(&mut hasher);

    //     let image_hash = hasher.finish();

    //     // we can unwrap Option<T> here as if 
    //     // "self.image_handler.image_loaded" is true image data should exist.
    //     match image.image_data.lock().unwrap().as_ref()
    //         .expect("Image data was not present! This is a logic error on our side, please report it.") {
    //         ImageData::Pixels((pixels, (width, height), image_colour_type)) => {
    //             let texture = match &self.egui_image_texture {
    //                 Some(texture) => texture,
    //                 None => {
    //                     debug!("Taking image texture and uploading it to the GPU with egui...");

    //                     let image_size = [*width as usize, *height as usize];

    //                     self.egui_image_texture = Some(
    //                         ctx.load_texture(
    //                             "image",
    //                             match image_colour_type {
    //                                 ImageColourType::Grey | ImageColourType::GreyAlpha => {
    //                                     debug!("Rendering image as grey scale egui texture...");
    //                                     egui::ColorImage::from_gray(
    //                                         image_size, pixels
    //                                     )
    //                                 },
    //                                 ImageColourType::RGB => {
    //                                     debug!("Rendering image as rgb egui texture...");
    //                                     egui::ColorImage::from_rgb(
    //                                         image_size, pixels
    //                                     )
    //                                 },
    //                                 ImageColourType::RGBA => {
    //                                     debug!("Rendering image as rgba egui texture...");
    //                                     egui::ColorImage::from_rgba_unmultiplied(
    //                                         image_size, pixels
    //                                     )
    //                                 },
    //                             },
    //                             TextureOptions {
    //                                 magnification: TextureFilter::Linear,
    //                                 minification: TextureFilter::Linear,
    //                                 wrap_mode: TextureWrapMode::ClampToEdge,
    //                                 mipmap_mode: None,
    //                             }
    //                         )
    //                     );

    //                     self.egui_image_texture.as_ref().unwrap()
    //                 },
    //             };

    //             egui::Image::from_texture(texture)
    //         },
    //         ImageData::StaticBytes(bytes) => {
    //             egui::Image::from_bytes(
    //                 format!("bytes://{}.{:#}", image_hash, image.image_format),
    //                 bytes.clone() // we can clone here without turning into a java application as we're using arc
    //             )
    //         },
    //     }
    // }

    /// Method that handles choosing which type of modifications 
    /// should be done to the image at this time. It decides that on a number of various factors, 
    /// like image optimizations applied by the user, monitor size, zoom factor and etc.
    fn get_image_modifications(&mut self, monitor_size: &MonitorSize) -> HashSet<ImageModifications> {
        let mut image_modifications = HashSet::new();

        let image = self.image.as_ref();

        if let Some(image) = image {
            // SVG and GIFs should not get image modifications.
            if let ImageFormat::Svg | ImageFormat::Gif = image.image_format {
                return image_modifications;
            }

            // the reason why we don't just loop over self.image_optimizations 
            // is because I need to make absolute sure I'm doing these checks in this exact order.

            if let Some(ImageOptimizations::MonitorDownsampling(marginal_allowance)) = self.image_optimizations.get(
                &ImageOptimizations::MonitorDownsampling(u32::default())
            ) {
                let (width, height) = get_monitor_downsampling_size(
                    marginal_allowance, monitor_size
                );

                // If the image is a lot bigger than the user's 
                // monitor then apply monitor downsample, if not we shouldn't.
                if image.image_size.0 > width && image.image_size.1 > height {
                    self.monitor_downsampling_required = true;

                    debug!(
                        "Image is significantly bigger than system's \
                        display monitor so monitor downsampling will be applied..."
                    );

                    let image_size = (image.image_size.0, image.image_size.1);

                    debug!(
                        "Image Size: {} x {}", image_size.0, image_size.1
                    );

                    let (monitor_width, monitor_height) = monitor_size.get();

                    debug!(
                        "Display (Monitor) Size: {} x {}", monitor_width, monitor_height
                    );

                    debug!(
                        "Display + Monitor Downsample Marginal Allowance ({}): {} x {}",
                        marginal_allowance, width, height
                    );

                    image_modifications.replace(ImageModifications::Resize((width, height)));
                }
            }

            // TODO: handle up and down dyn sampling options.
            // NOTE: I think I might just add "down" as a bool tbh, you'll never want 
            // upsampling to be disabled if you choose to enable dyn sampling in the first place.
            if let Some(ImageOptimizations::DynamicSampling(up, down)) = self.image_optimizations.get(
                &ImageOptimizations::DynamicSampling(bool::default(), bool::default())
            ) {
                let new_resolution = self.dynamic_sampling_new_resolution;
                let old_resolution = self.dynamic_sampling_old_resolution;

                if !(new_resolution == old_resolution) {
                    debug!(
                        "User zoomed far enough into downsampled image, \
                        dynamic sampling will be performed... \n\t({:?} -> {:?})",
                        old_resolution, new_resolution
                    );

                    if !(new_resolution.0 == image.image_size.0 && new_resolution.1 == image.image_size.1) {
                        image_modifications.replace(
                            ImageModifications::Resize(new_resolution)
                        );

                        self.dynamic_sampling_old_resolution = new_resolution;
                    } else {
                        debug!(
                            "Not applying resize mod for dynamic sampling as \
                            dynamic sampling is requesting the full resolution already!"
                        );

                        image_modifications.remove(
                            &ImageModifications::Resize(new_resolution)
                        );

                        self.dynamic_sampling_old_resolution = (
                            image.image_size.0,
                            image.image_size.1
                        );
                    }
                }
            }
        };

        image_modifications
    }
}
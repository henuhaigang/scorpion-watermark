use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::NSString;

pub fn load_heic(path: &str) -> Result<image::DynamicImage, String> {
    unsafe {
        let ns_path = NSString::from_str(path);
        let ns_url: *mut AnyObject = msg_send![class!(NSURL), fileURLWithPath: &*ns_path];

        let image_source: *mut AnyObject = msg_send![class!(CGImageSource), imageSourceWithURL: ns_url options: std::ptr::null_mut::<AnyObject>()];

        if image_source.is_null() {
            return Err("Failed to create CGImageSource from HEIC file".to_string());
        }

        let cg_image: *mut AnyObject = msg_send![image_source, createImageAtIndex: 0u64 options: std::ptr::null_mut::<AnyObject>()];

        if cg_image.is_null() {
            return Err("Failed to create CGImage from HEIC".to_string());
        }

        let width: usize = msg_send![cg_image, width];
        let height: usize = msg_send![cg_image, height];

        let color_space: *mut AnyObject = msg_send![class!(CGColorSpace), deviceRGBColorSpace];
        let mut raw_data = vec![0u8; width * height * 4];

        let context: *mut AnyObject = msg_send![class!(CGBitmapContext), newWithData: raw_data.as_mut_ptr() width: width height: height bitsPerComponent: 8u32 bytesPerRow: width * 4usize colorSpace: color_space bitmapInfo: 0x00000001u32];

        if context.is_null() {
            return Err("Failed to create CGBitmapContext".to_string());
        }

        let _: () = msg_send![context, drawImage: cg_image inRectX: 0.0f64 inRectY: 0.0f64 inRectWidth: width as f64 inRectHeight: height as f64];

        let rgba_image = image::RgbaImage::from_raw(width as u32, height as u32, raw_data)
            .ok_or("Failed to create RgbaImage from raw data")?;

        Ok(image::DynamicImage::ImageRgba8(rgba_image))
    }
}

pub fn save_heic(image: &image::DynamicImage, path: &str) -> Result<(), String> {
    unsafe {
        let rgba = image.to_rgba8();
        let (width, height) = (rgba.width() as usize, rgba.height() as usize);

        let color_space: *mut AnyObject = msg_send![class!(CGColorSpace), deviceRGBColorSpace];
        let data_provider: *mut AnyObject = msg_send![class!(CGDataProvider), newWithData: rgba.as_ptr() length: width * height * 4usize];

        if data_provider.is_null() {
            return Err("Failed to create CGDataProvider".to_string());
        }

        let cg_image: *mut AnyObject = msg_send![class!(CGImage), newWithWidth: width height: height bitsPerComponent: 8u32 bitsPerPixel: 32u32 bytesPerRow: width * 4usize colorSpace: color_space bitmapInfo: 0x00000001u32 provider: data_provider shouldInterpolate: false intent: 0i32];

        if cg_image.is_null() {
            return Err("Failed to create CGImage".to_string());
        }

        let ns_path = NSString::from_str(path);
        let ns_url: *mut AnyObject = msg_send![class!(NSURL), fileURLWithPath: &*ns_path];

        let heic_uti = NSString::from_str("public.heic");
        let image_destination: *mut AnyObject = msg_send![class!(CGImageDestination), newWithURL: ns_url type: &*heic_uti count: 1u64 options: std::ptr::null_mut::<AnyObject>()];

        if image_destination.is_null() {
            return Err("Failed to create CGImageDestination for HEIC".to_string());
        }

        let _: () = msg_send![image_destination, addImage: cg_image fromSource: image_destination properties: std::ptr::null_mut::<AnyObject>()];

        let success: bool = msg_send![image_destination, finalize];

        if !success {
            return Err("Failed to finalize HEIC image".to_string());
        }

        Ok(())
    }
}

extern "C" {
    static kUTTypeHEIC: *const AnyObject;
}

#[link(name = "ImageIO", kind = "framework")]
extern "C" {}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {}

#[link(name = "UniformTypeIdentifiers", kind = "framework")]
extern "C" {}

use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::DialogExt;
use serde::{Deserialize, Serialize};
use image::GenericImageView;
use tokio::sync::oneshot;

#[cfg(target_os = "ios")]
use std::{thread, time::Duration, ffi::{CStr, CString}, os::raw::c_char, sync::{Mutex, OnceLock}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageSource {
    Gallery,
    Camera,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickImageOptions {
    pub source: ImageSource,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub quality: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickedImage {
    pub path: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

// Global storage for the oneshot sender on iOS (accessible from any thread)
#[cfg(target_os = "ios")]
static IOS_PICKER_SENDER: OnceLock<Mutex<Option<oneshot::Sender<Result<PickedImage, String>>>>> = OnceLock::new();

#[tauri::command]
pub async fn test_ipc() -> Result<String, String> {
    eprintln!("[test_ipc] Called!");
    Ok("IPC works!".to_string())
}

#[tauri::command]
pub async fn pick_image<R: Runtime>(
    app: AppHandle<R>,
    options: PickImageOptions,
) -> Result<PickedImage, String> {
    eprintln!("[pick_image] >>> Called with source: {:?}", options.source);
    
    #[cfg(target_os = "ios")]
    {
        eprintln!("[pick_image] iOS target detected, calling pick_image_ios");
        let result = pick_image_ios(options).await;
        eprintln!("[pick_image] <<< pick_image_ios returned: {:?}", result);
        return result;
    }
    
    #[cfg(not(target_os = "ios"))]
    {
        eprintln!("[pick_image] Non-iOS target, calling pick_image_fallback");
        let result = pick_image_fallback(app, options).await;
        eprintln!("[pick_image] <<< pick_image_fallback returned: {:?}", result);
        return result;
    }
}

#[cfg(target_os = "ios")]
async fn pick_image_ios(options: PickImageOptions) -> Result<PickedImage, String> {
    use std::thread;
    use std::time::Duration;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    
    let source_str = match options.source {
        ImageSource::Gallery => "gallery",
        ImageSource::Camera => "camera",
        ImageSource::File => "file",
    };
    
    let max_width = options.max_width.unwrap_or(4096) as i32;
    let max_height = options.max_height.unwrap_or(4096) as i32;
    let quality = options.quality.unwrap_or(90) as i32;
    
    let (tx, rx) = oneshot::channel::<Result<PickedImage, String>>();
    
    // Store the sender in global storage for the callback to use
    IOS_PICKER_SENDER.get_or_init(|| Mutex::new(None))
        .lock().unwrap()
        .replace(tx);
    
    // Call the Swift C function on a background thread
    // The Swift function handles dispatching to main thread internally
    thread::spawn(move || {
        let source_c = CString::new(source_str).unwrap();
        
        // Declare the external C function
        extern "C" {
            fn scorpion_pick_image(
                source: *const c_char,
                max_width: i32,
                max_height: i32,
                quality: i32,
                callback: extern "C" fn(*const c_char),
            );
        }
        
        // Define the callback as a static function (no captures)
        extern "C" fn callback(json_ptr: *const c_char) {
            let json_str = unsafe { CStr::from_ptr(json_ptr).to_string_lossy().into_owned() };
            
            eprintln!("[pick_image_ios] Swift callback received: {}", json_str);
            
            // Use global storage to send the result
            let sender = IOS_PICKER_SENDER.get().expect("IOS_PICKER_SENDER not initialized");
            let mut guard = sender.lock().unwrap();
            if let Some(tx) = guard.take() {
                // Parse the JSON result
                if json_str.is_empty() || json_str == "{}" || json_str.contains("\"cancelled\": true") {
                    eprintln!("[pick_image_ios] User cancelled");
                    let _ = tx.send(Err("User cancelled".to_string()));
                    return;
                }
                
                // Simple JSON parsing for our known format
                if json_str.contains("\"error\"") {
                    // Extract error message
                    if let Some(start) = json_str.find("\"error\": \"") {
                        let start = start + 10;
                        if let Some(end) = json_str[start..].find('"') {
                            let error = &json_str[start..start + end];
                            eprintln!("[pick_image_ios] Error from Swift: {}", error);
                            let _ = tx.send(Err(error.to_string()));
                            return;
                        }
                    }
                    eprintln!("[pick_image_ios] Unknown error from Swift");
                    let _ = tx.send(Err("Unknown error".to_string()));
                    return;
                }
                
                // Parse success result
                let mut path = "";
                let mut mime_type = "";
                let mut width = 0u32;
                let mut height = 0u32;
                
                if let Some(p) = extract_json_field(&json_str, "path") {
                    path = p;
                }
                if let Some(m) = extract_json_field(&json_str, "mimeType") {
                    mime_type = m;
                }
                if let Some(w) = extract_json_field(&json_str, "width") {
                    width = w.parse().unwrap_or(0);
                }
                if let Some(h) = extract_json_field(&json_str, "height") {
                    height = h.parse().unwrap_or(0);
                }
                
                if !path.is_empty() {
                    eprintln!("[pick_image_ios] Success: path={}, mime={}, {}x{}", path, mime_type, width, height);
                    // Verify file exists
                    if std::path::Path::new(path).exists() {
                        eprintln!("[pick_image_ios] File exists at path");
                    } else {
                        eprintln!("[pick_image_ios] WARNING: File does NOT exist at path!");
                    }
                    let _ = tx.send(Ok(PickedImage {
                        path: path.to_string(),
                        mime_type: mime_type.to_string(),
                        width,
                        height,
                    }));
                } else {
                    eprintln!("[pick_image_ios] Invalid result - empty path");
                    let _ = tx.send(Err("Invalid result".to_string()));
                }
            } else {
                eprintln!("[pick_image_ios] No sender in global storage!");
            }
        }
        
        // Call the Swift function
        unsafe {
            scorpion_pick_image(source_c.as_ptr(), max_width, max_height, quality, callback);
        }
    });

    // Wait for result with timeout
    match tokio::time::timeout(Duration::from_secs(120), rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("Timeout waiting for image picker".to_string()),
    }
}

fn extract_json_field<'a>(json: &'a str, field: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\": ", field);
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        let remaining = &json[value_start..];
        if remaining.starts_with('"') {
            // String value
            if let Some(end) = remaining[1..].find('"') {
                return Some(&remaining[1..1 + end]);
            }
        } else {
            // Number value
            let end = remaining.find(|c: char| c == ',' || c == '}').unwrap_or(remaining.len());
            return Some(&remaining[..end].trim());
        }
    }
    None
}

async fn pick_image_fallback<R: Runtime>(
    app: AppHandle<R>,
    options: PickImageOptions,
) -> Result<PickedImage, String> {
    let (tx, rx) = oneshot::channel();
    
    let filter_name = match options.source {
        ImageSource::Camera => "Images",
        _ => "Images",
    };
    
    app.dialog()
        .file()
        .add_filter(filter_name, &["jpg", "jpeg", "png", "gif", "webp", "heic"])
        .pick_file(move |file_path| {
            let path_str = file_path.map(|p| p.to_string()).unwrap_or_default();
            if path_str.is_empty() {
                let _ = tx.send(Err("User cancelled".to_string()));
                return;
            }
            
            let mime_type = match path_str.split('.').last().unwrap_or("") {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "heic" => "image/heic",
                _ => "application/octet-stream",
            };

            let img = match image::ImageReader::open(&path_str) {
                Ok(reader) => match reader.decode() {
                    Ok(img) => img,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Decode: {}", e)));
                        return;
                    }
                },
                Err(e) => {
                    let _ = tx.send(Err(format!("Open image: {}", e)));
                    return;
                }
            };
            let (width, height) = img.dimensions();

            let _ = tx.send(Ok(PickedImage {
                path: path_str,
                mime_type: mime_type.to_string(),
                width,
                height,
            }));
        });

    // Wait for the result with a timeout
    match tokio::time::timeout(std::time::Duration::from_secs(60), rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => Err(e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
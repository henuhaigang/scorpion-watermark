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
    
    #[cfg(target_os = "android")]
    {
        eprintln!("[pick_image] Android target detected, calling pick_image_android");
        let result = pick_image_android(app, options).await;
        eprintln!("[pick_image] <<< pick_image_android returned: {:?}", result);
        return result;
    }
    
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        eprintln!("[pick_image] Non-iOS/Android target, calling pick_image_fallback");
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

#[cfg(target_os = "android")]
async fn pick_image_android<R: Runtime>(
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
            
            // On Android, the path might be a content:// URI
            // We need to copy it to a temporary file first
            let (resolved_path, mime_type) = if path_str.starts_with("content://") {
                // Use JNI to resolve content URI to file
                match resolve_content_uri(&path_str) {
                    Ok((p, m)) => (p, m),
                    Err(e) => {
                        let _ = tx.send(Err(format!("Resolve content URI: {}", e)));
                        return;
                    }
                }
            } else {
                let mime_type = match path_str.split('.').last().unwrap_or("") {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "gif" => "image/gif",
                    "webp" => "image/webp",
                    "heic" => "image/heic",
                    _ => "application/octet-stream",
                };
                (path_str, mime_type.to_string())
            };

            let img = match image::ImageReader::open(&resolved_path) {
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
                path: resolved_path,
                mime_type,
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

#[cfg(target_os = "android")]
fn resolve_content_uri(uri_str: &str) -> Result<(String, String), String> {
    use jni::{JavaVM, objects::{JObject, JValue, JValueGen}};
    
    // Get the JavaVM from ndk_context
    let vm_ptr = unsafe { ndk_context::android_context().vm() };
    let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _) }.map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    // Get the context
    let context = unsafe { ndk_context::android_context().context() };
    let context_obj = unsafe { JObject::from_raw(context as *mut _) };
    
    // Get ContentResolver
    let content_resolver = env.call_method(
        &context_obj,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[],
    ).map_err(|e| format!("getContentResolver failed: {}", e))?.l().map_err(|e| format!("getContentResolver not object: {}", e))?;
    
    // Get Uri class and parse the URI
    let uri_class = env.find_class("android/net/Uri").map_err(|e| format!("find Uri class: {}", e))?;
    let uri_string = env.new_string(uri_str).map_err(|e| format!("new string: {}", e))?;
    let uri = env.call_static_method(uri_class, "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[JValue::Object(&uri_string)]).map_err(|e| format!("parse URI: {}", e))?.l().map_err(|e| format!("parse URI not object: {}", e))?;
    
    // Open input stream
    let input_stream = env.call_method(
        &content_resolver,
        "openInputStream",
        "(Landroid/net/Uri;)Ljava/io/InputStream;",
        &[JValue::Object(&uri)],
    ).map_err(|e| format!("openInputStream failed: {}", e))?.l().map_err(|e| format!("openInputStream not object: {}", e))?;
    
    // Get MIME type to determine file extension
    let mime_type = env.call_method(
        &content_resolver,
        "getType",
        "(Landroid/net/Uri;)Ljava/lang/String;",
        &[JValue::Object(&uri)],
    ).map_err(|e| format!("getType failed: {}", e))?.l().map_err(|e| format!("getType not object: {}", e))?;
    let mime_type_str: String = if mime_type.is_null() {
        "image/jpeg".to_string()
    } else {
        env.get_string(&mime_type.into()).map_err(|e| format!("get mime string: {}", e))?.into()
    };
    
    // Determine file extension from MIME type
    let extension = match mime_type_str.as_str() {
        "image/png" => ".png",
        "image/gif" => ".gif",
        "image/webp" => ".webp",
        "image/heic" | "image/heif" => ".heic",
        _ => ".jpg", // default to jpg for image/jpeg and unknown
    };
    
    // Create temp file with proper extension
    let cache_dir = env.call_method(
        &context_obj,
        "getCacheDir",
        "()Ljava/io/File;",
        &[],
    ).map_err(|e| format!("getCacheDir failed: {}", e))?.l().map_err(|e| format!("getCacheDir not object: {}", e))?;
    
    let file_class = env.find_class("java/io/File").map_err(|e| format!("find File class: {}", e))?;
    let prefix = env.new_string("scorpion_").map_err(|e| format!("new prefix string: {}", e))?;
    let suffix = env.new_string(extension).map_err(|e| format!("new suffix string: {}", e))?;
    let temp_file = env.call_static_method(file_class, "createTempFile", "(Ljava/lang/String;Ljava/lang/String;Ljava/io/File;)Ljava/io/File;", &[JValue::Object(&prefix), JValue::Object(&suffix), JValue::Object(&cache_dir)]).map_err(|e| format!("createTempFile failed: {}", e))?.l().map_err(|e| format!("createTempFile not object: {}", e))?;
    
    // Get temp file path
    let temp_path = env.call_method(&temp_file, "getAbsolutePath", "()Ljava/lang/String;", &[]).map_err(|e| format!("getAbsolutePath failed: {}", e))?.l().map_err(|e| format!("getAbsolutePath not object: {}", e))?;
    let temp_path_str: String = env.get_string(&temp_path.into()).map_err(|e| format!("get string: {}", e))?.into();
    
    // Copy input stream to temp file
    let file_output_stream_class = env.find_class("java/io/FileOutputStream").map_err(|e| format!("find FileOutputStream: {}", e))?;
    let fos = env.new_object(file_output_stream_class, "(Ljava/io/File;)V", &[JValue::Object(&temp_file)]).map_err(|e| format!("new FileOutputStream: {}", e))?;
    
    // Copy bytes
    let mut buffer = [0i8; 8192];
    
    loop {
        let byte_array = env.new_byte_array(buffer.len() as i32).map_err(|e| format!("new byte array: {}", e))?;
        let bytes_read = env.call_method(&input_stream, "read", "([B)I", &[JValue::Object(&byte_array)]).map_err(|e| format!("read failed: {}", e))?.i().map_err(|e| format!("read not int: {}", e))?;
        
        if bytes_read <= 0 {
            break;
        }
        
        env.get_byte_array_region(&byte_array, 0, &mut buffer[..bytes_read as usize]).map_err(|e| format!("get byte array region: {}", e))?;
        
        env.call_method(&fos, "write", "([BII)V", &[JValue::Object(&byte_array), JValue::Int(0), JValue::Int(bytes_read)]).map_err(|e| format!("write failed: {}", e))?;
    }
    
    // Close streams
    env.call_method(&input_stream, "close", "()V", &[]).map_err(|e| format!("close input stream: {}", e))?;
    env.call_method(&fos, "close", "()V", &[]).map_err(|e| format!("close fos: {}", e))?;
    
    Ok((temp_path_str, mime_type_str))
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
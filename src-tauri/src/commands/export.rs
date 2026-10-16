use crate::core::config::{OutputFormat, WatermarkConfig};
use crate::core::pdf_watermark::add_pdf_watermark;
use crate::core::pipeline::process_image;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use std::path::Path;
use percent_encoding::percent_decode_str;
use uuid;

#[cfg(target_os = "android")]
use jni::{JavaVM, objects::{JObject, JValue}};
#[cfg(target_os = "android")]
use std::io::{Read, Write};

fn url_to_path(url: &str) -> String {
    let s = url.strip_prefix("file://").unwrap_or(url);
    let decoded = percent_decode_str(s).decode_utf8_lossy();
    decoded.into_owned()
}

#[cfg(target_os = "android")]
fn write_to_content_uri(content_uri: &str, src_path: &Path) -> Result<(), String> {
    use jni::{JavaVM, objects::JValue};
    
    let vm_ptr = unsafe { ndk_context::android_context().vm() };
    let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _) }.map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    let context = unsafe { ndk_context::android_context().context() };
    let context_obj = unsafe { JObject::from_raw(context as *mut _) };
    
    // Get ContentResolver
    let content_resolver = env.call_method(
        &context_obj,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[],
    ).map_err(|e| format!("getContentResolver failed: {}", e))?.l().map_err(|e| format!("getContentResolver not object: {}", e))?;
    
    // Parse URI
    let uri_class = env.find_class("android/net/Uri").map_err(|e| format!("find Uri class: {}", e))?;
    let uri_string = env.new_string(content_uri).map_err(|e| format!("new string: {}", e))?;
    let uri = env.call_static_method(uri_class, "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[JValue::Object(&uri_string)]).map_err(|e| format!("parse URI: {}", e))?.l().map_err(|e| format!("parse URI not object: {}", e))?;
    
    // Open output stream
    let output_stream = env.call_method(
        &content_resolver,
        "openOutputStream",
        "(Landroid/net/Uri;)Ljava/io/OutputStream;",
        &[JValue::Object(&uri)],
    ).map_err(|e| format!("openOutputStream failed: {}", e))?.l().map_err(|e| format!("openOutputStream not object: {}", e))?;
    
    // Read source file and write to output stream
    let mut src_file = std::fs::File::open(src_path).map_err(|e| format!("open src file: {}", e))?;
    let mut u8_buffer = [0u8; 8192];
    let mut i8_buffer = [0i8; 8192];
    
    loop {
        let bytes_read = src_file.read(&mut u8_buffer).map_err(|e| format!("read src file: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        
        // Convert u8 to i8 for JNI
        for i in 0..bytes_read {
            i8_buffer[i] = u8_buffer[i] as i8;
        }
        
        // Create byte array and fill with data
        let byte_array = env.new_byte_array(bytes_read as i32).map_err(|e| format!("new byte array: {}", e))?;
        env.set_byte_array_region(&byte_array, 0, &i8_buffer[..bytes_read]).map_err(|e| format!("set byte array region: {}", e))?;
        env.call_method(&output_stream, "write", "([BII)V", &[JValue::Object(&byte_array), JValue::Int(0), JValue::Int(bytes_read as i32)]).map_err(|e| format!("write failed: {}", e))?;
    }
    
    // Close streams
    env.call_method(&output_stream, "close", "()V", &[]).map_err(|e| format!("close output stream: {}", e))?;
    
    Ok(())
}

#[cfg(target_os = "android")]
fn save_to_mediastore(src_path: &Path, display_name: &str, extension: &str) -> Result<(), String> {
    use jni::{JavaVM, objects::{JObject, JValue}};
    
    eprintln!("[save_to_mediastore] src_path: {:?}, display_name: {}, extension: {}", src_path, display_name, extension);
    
    let vm_ptr = unsafe { ndk_context::android_context().vm() };
    let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _) }.map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    let context = unsafe { ndk_context::android_context().context() };
    let context_obj = unsafe { JObject::from_raw(context as *mut _) };
    
    // Get ContentResolver
    let context_class = env.get_object_class(&context_obj).map_err(|e| format!("get Context class: {}", e))?;
    let content_resolver = env.call_method(
        &context_obj,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[],
    ).map_err(|e| format!("getContentResolver failed: {}", e))?.l().map_err(|e| format!("getContentResolver not object: {}", e))?;
    
    // Determine MIME type
    let mime_type = match extension {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "heic" => "image/heic",
        _ => "image/jpeg",
    };
    
// Create ContentValues for the new image
    let content_values_class = env.find_class("android/content/ContentValues").map_err(|e| format!("find ContentValues class: {}", e))?;
    let content_values = env.new_object(content_values_class, "()V", &[]).map_err(|e| format!("new ContentValues: {}", e))?;
    
    // Set required fields directly
    let display_name_key = env.new_string("_display_name").map_err(|e| format!("new string: {}", e))?;
    let mime_type_key = env.new_string("mime_type").map_err(|e| format!("new string: {}", e))?;
    let relative_path_key = env.new_string("relative_path").map_err(|e| format!("new string: {}", e))?;
    let display_name_obj = env.new_string(display_name).map_err(|e| format!("new string: {}", e))?;
    let mime_type_obj = env.new_string(mime_type).map_err(|e| format!("new string: {}", e))?;
    let relative_path_obj = env.new_string("Pictures/ScorpionWatermark").map_err(|e| format!("new string: {}", e))?;
    
    env.call_method(&content_values, "put", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&display_name_key), JValue::Object(&display_name_obj)]).map_err(|e| format!("put display_name: {}", e))?;
    env.call_method(&content_values, "put", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&mime_type_key), JValue::Object(&mime_type_obj)]).map_err(|e| format!("put mime_type: {}", e))?;
    env.call_method(&content_values, "put", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&relative_path_key), JValue::Object(&relative_path_obj)]).map_err(|e| format!("put relative_path: {}", e))?;
    
    // Get MediaStore.Images.Media.EXTERNAL_CONTENT_URI
    let mediastore_class = env.find_class("android/provider/MediaStore$Images$Media").map_err(|e| format!("find MediaStore class: {}", e))?;
    let external_content_uri = env.get_static_field(mediastore_class, "EXTERNAL_CONTENT_URI", "Landroid/net/Uri;").map_err(|e| format!("get EXTERNAL_CONTENT_URI: {}", e))?.l().map_err(|e| format!("EXTERNAL_CONTENT_URI not object: {}", e))?;
    
    // Insert into MediaStore
    let content_resolver_class = env.get_object_class(&content_resolver).map_err(|e| format!("get ContentResolver class: {}", e))?;
    let insert_method = env.get_method_id(&content_resolver_class, "insert", "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;").map_err(|e| format!("get insert method: {}", e))?;
    
    let result_uri = env.call_method(&content_resolver, "insert", "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;", &[JValue::Object(&external_content_uri), JValue::Object(&content_values)])
        .map_err(|e| format!("insert failed: {}", e))?.l().map_err(|e| format!("insert result not object: {}", e))?;
    
    if result_uri.is_null() {
        return Err("Failed to insert into MediaStore".to_string());
    }
    
    // Open output stream for the new URI
    let output_stream = env.call_method(
        &content_resolver,
        "openOutputStream",
        "(Landroid/net/Uri;)Ljava/io/OutputStream;",
        &[JValue::Object(&result_uri)],
    ).map_err(|e| format!("openOutputStream failed: {}", e))?.l().map_err(|e| format!("openOutputStream not object: {}", e))?;
    
    // Read source file and write to output stream
    let mut src_file = std::fs::File::open(src_path).map_err(|e| format!("open src file: {}", e))?;
    let mut u8_buffer = [0u8; 8192];
    let mut i8_buffer = [0i8; 8192];
    
    loop {
        let bytes_read = src_file.read(&mut u8_buffer).map_err(|e| format!("read src file: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        
        // Convert u8 to i8 for JNI
        for i in 0..bytes_read {
            i8_buffer[i] = u8_buffer[i] as i8;
        }
        
        // Create byte array and fill with data
        let byte_array = env.new_byte_array(bytes_read as i32).map_err(|e| format!("new byte array: {}", e))?;
        env.set_byte_array_region(&byte_array, 0, &i8_buffer[..bytes_read]).map_err(|e| format!("set byte array region: {}", e))?;
        env.call_method(&output_stream, "write", "([BII)V", &[JValue::Object(&byte_array), JValue::Int(0), JValue::Int(bytes_read as i32)]).map_err(|e| format!("write failed: {}", e))?;
    }
    
    // Close streams
    env.call_method(&output_stream, "close", "()V", &[]).map_err(|e| format!("close output stream: {}", e))?;
    
    // Scan the file so it appears in gallery
    scan_media_file(&display_name, extension)?;
    
    Ok(())
}

#[cfg(target_os = "android")]
fn get_android_pictures_dir() -> Result<std::path::PathBuf, String> {
    use jni::{JavaVM, objects::JValue};
    
    let vm_ptr = unsafe { ndk_context::android_context().vm() };
    let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _) }.map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    // Use Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES)
    // This returns the public Pictures directory
    let env_class = env.find_class("android/os/Environment").map_err(|e| format!("find Environment class: {}", e))?;
    let pictures_dir_field = env.get_static_field(&env_class, "DIRECTORY_PICTURES", "Ljava/lang/String;").map_err(|e| format!("get DIRECTORY_PICTURES: {}", e))?.l().map_err(|e| format!("DIRECTORY_PICTURES not object: {}", e))?;
    
    // Use getExternalStoragePublicDirectory - call with method name string directly
    let public_dir = env.call_static_method(env_class, "getExternalStoragePublicDirectory", "(Ljava/lang/String;)Ljava/io/File;", &[JValue::Object(&pictures_dir_field)]).map_err(|e| format!("getExternalStoragePublicDirectory failed: {}", e))?.l().map_err(|e| format!("getExternalStoragePublicDirectory not object: {}", e))?;
    
    // Get absolute path
    let file_class = env.find_class("java/io/File").map_err(|e| format!("find File class: {}", e))?;
    let get_absolute_path = env.get_method_id(file_class, "getAbsolutePath", "()Ljava/lang/String;").map_err(|e| format!("get getAbsolutePath: {}", e))?;
    let path_str = env.call_method(&public_dir, "getAbsolutePath", "()Ljava/lang/String;", &[]).map_err(|e| format!("getAbsolutePath failed: {}", e))?;
    let path_str = path_str.l().map_err(|e| format!("getAbsolutePath not object: {}", e))?;
    let path_string: String = env.get_string(&path_str.into()).map_err(|e| format!("get string: {}", e))?.into();
    
    Ok(std::path::PathBuf::from(path_string))
}

#[cfg(target_os = "android")]
fn get_android_cache_dir() -> Result<std::path::PathBuf, String> {
    use jni::{JavaVM, objects::JValue};
    
    let vm_ptr = unsafe { ndk_context::android_context().vm() };
    let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _) }.map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    let context = unsafe { ndk_context::android_context().context() };
    let context_obj = unsafe { JObject::from_raw(context as *mut _) };
    
    // Get Context.getCacheDir()
    let context_class = env.get_object_class(&context_obj).map_err(|e| format!("get Context class: {}", e))?;
    let get_cache_dir = env.get_method_id(context_class, "getCacheDir", "()Ljava/io/File;").map_err(|e| format!("get getCacheDir: {}", e))?;
    let cache_dir = env.call_method(&context_obj, "getCacheDir", "()Ljava/io/File;", &[]).map_err(|e| format!("getCacheDir failed: {}", e))?;
    let cache_dir = cache_dir.l().map_err(|e| format!("getCacheDir not object: {}", e))?;
    
    // Get absolute path
    let file_class = env.find_class("java/io/File").map_err(|e| format!("find File class: {}", e))?;
    let get_absolute_path = env.get_method_id(file_class, "getAbsolutePath", "()Ljava/lang/String;").map_err(|e| format!("get getAbsolutePath: {}", e))?;
    let path_str = env.call_method(&cache_dir, "getAbsolutePath", "()Ljava/lang/String;", &[]).map_err(|e| format!("getAbsolutePath failed: {}", e))?;
    let path_str = path_str.l().map_err(|e| format!("getAbsolutePath not object: {}", e))?;
    let path_string: String = env.get_string(&path_str.into()).map_err(|e| format!("get string: {}", e))?.into();
    
    Ok(std::path::PathBuf::from(path_string))
}

#[cfg(target_os = "android")]
fn scan_media_file(file_path: &str, extension: &str) -> Result<(), String> {
    use jni::{JavaVM, objects::{JObject, JValue}};
    
    let vm_ptr = unsafe { ndk_context::android_context().vm() };
    let vm = unsafe { JavaVM::from_raw(vm_ptr as *mut _) }.map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    let context = unsafe { ndk_context::android_context().context() };
    let context_obj = unsafe { JObject::from_raw(context as *mut _) };
    
    // Find classes first
    let media_scanner_class = env.find_class("android/media/MediaScannerConnection").map_err(|e| format!("find MediaScannerConnection: {}", e))?;
    let string_class = env.find_class("java/lang/String").map_err(|e| format!("find String class: {}", e))?;
    
    // Create arrays
    let paths_array = env.new_object_array(1, &string_class, unsafe { JObject::from_raw(std::ptr::null_mut()) }).map_err(|e| format!("new paths array: {}", e))?;
    let path_obj = env.new_string(file_path).map_err(|e| format!("new file path: {}", e))?;
    env.set_object_array_element(&paths_array, 0, &path_obj).map_err(|e| format!("set array element: {}", e))?;
    
    let mime_types_array = env.new_object_array(1, &string_class, unsafe { JObject::from_raw(std::ptr::null_mut()) }).map_err(|e| format!("new mime types array: {}", e))?;
    let mime_type = match extension {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "heic" => "image/heic",
        _ => "image/jpeg",
    };
    let mime_type_obj = env.new_string(mime_type).map_err(|e| format!("new mime type: {}", e))?;
    env.set_object_array_element(&mime_types_array, 0, &mime_type_obj).map_err(|e| format!("set mime type array element: {}", e))?;
    
    let null_obj = unsafe { JObject::from_raw(std::ptr::null_mut()) };
    
    env.call_static_method(
        media_scanner_class,
        "scanFile",
        "(Landroid/content/Context;[Ljava/lang/String;[Ljava/lang/String;Landroid/media/MediaScannerConnection$OnScanCompletedListener;)V",
        &[JValue::Object(&context_obj), JValue::Object(&paths_array), JValue::Object(&mime_types_array), JValue::Object(&null_obj)],
    ).map_err(|e| format!("scanFile failed: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn export_file(
    app: tauri::AppHandle,
    input_path: String,
    config: WatermarkConfig,
) -> Result<(), String> {
    let input_path_buf = Path::new(&input_path);
    let input_stem = input_path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("watermarked");
    let input_ext = input_path_buf
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let target_ext = match config.output.format {
        OutputFormat::Jpeg => "jpg",
        OutputFormat::Png => "png",
        OutputFormat::Heic => "heic",
        OutputFormat::SameAsInput => {
            if input_ext == "heic" || input_ext == "heif" {
                "heic"
            } else if input_ext == "png" {
                "png"
            } else if input_ext == "pdf" {
                "pdf"
            } else {
                "jpg"
            }
        }
    };

    let default_name = format!("{}_watermarked.{}", input_stem, target_ext);
    
    eprintln!("[export_file] input_path: {}, target_ext: {}, default_name: {}", input_path, target_ext, default_name);

    #[cfg(target_os = "android")]
    {
        use std::sync::mpsc;
        
        // Use Tauri's path API to get the cache directory (no JNI needed)
        let cache_dir = app.path().app_cache_dir()
            .map_err(|e| format!("Failed to get cache dir: {}", e))?;
        let temp_path = cache_dir.join(format!("{}.{}", uuid::Uuid::new_v4(), target_ext));
        eprintln!("[export_file] cache_dir: {:?}, temp_path: {:?}", cache_dir, temp_path);
        
        // Process the image (CPU-bound, safe on background thread)
        if input_ext == "pdf" {
            add_pdf_watermark(&input_path, temp_path.to_str().unwrap(), &config.visible)?;
        } else {
            eprintln!("[export_file] calling process_image");
            process_image(&input_path, temp_path.to_str().unwrap(), &config)?;
            eprintln!("[export_file] process_image done");
        }
        
        // Dispatch save_to_mediastore to the main thread (JNI must run on main thread)
        let (tx, rx) = mpsc::channel::<Result<(), String>>();
        let temp_path_clone = temp_path.clone();
        let default_name_clone = default_name.clone();
        let target_ext_clone = target_ext.to_string();
        
        eprintln!("[export_file] dispatching save_to_mediastore to main thread");
        app.run_on_main_thread(move || {
            let result = save_to_mediastore(&temp_path_clone, &default_name_clone, &target_ext_clone);
            eprintln!("[export_file] save_to_mediastore result: {:?}", result.as_ref().err());
            let _ = tx.send(result);
        }).map_err(|e| format!("Failed to dispatch to main thread: {}", e))?;
        
        // Wait for the result from the main thread
        let result = rx.recv().map_err(|e| format!("Failed to receive result from main thread: {}", e))?;
        result?;
        eprintln!("[export_file] save_to_mediastore done");
        
        // Clean up temp file
        let _ = std::fs::remove_file(temp_path);
        
        Ok(())
    }
    
    #[cfg(not(target_os = "android"))]
    {
        let output_path = app
            .dialog()
            .file()
            .set_file_name(&default_name)
            .add_filter("Image", &[target_ext])
            .blocking_save_file();

        let output_path = match output_path {
            Some(path) => url_to_path(&path.to_string()),
            None => return Err("No output path selected".to_string()),
        };

        // Ensure the output path has the correct target extension
        let output_path = {
            let path_buf = Path::new(&output_path);
            let parent = path_buf.parent().unwrap_or_else(|| Path::new("."));
            let user_stem = path_buf.file_stem().and_then(|s| s.to_str()).unwrap_or(&default_name[..default_name.len()-target_ext.len()-1]);
            parent.join(format!("{}.{}", user_stem, target_ext)).to_string_lossy().to_string()
        };

        if input_ext == "pdf" {
            add_pdf_watermark(&input_path, &output_path, &config.visible)?;
        } else {
            process_image(&input_path, &output_path, &config)?;
        }
        
        Ok(())
    }
}

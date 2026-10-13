import Foundation
import PhotosUI
import UIKit
import UniformTypeIdentifiers

@objc public class ImagePicker: NSObject {
    private var continuation: CheckedContinuation<[String: Any]?, Error>?
    private var pickerType: String = "gallery"
    private weak var presentingViewController: UIViewController?
    private var isPresentingPicker = false
    
    // Static dictionary to retain pickers until callback
    static var retainedPickers: [UnsafeRawPointer: ImagePicker] = [:]
    
    // Associated object keys (static to ensure stable addresses)
    private static var completionKey = "completionKey"
    private static var maxWidthKey = "maxWidthKey"
    private static var maxHeightKey = "maxHeightKey"
    private static var qualityKey = "qualityKey"
    private static var isPhotoLibraryKey = "isPhotoLibraryKey"
    
    func log(_ message: String) {
        NSLog("[ImagePicker] %@", message)
    }
    
    @objc public func pickImage(
        source: String,
        maxWidth: Int = 4096,
        maxHeight: Int = 4096,
        quality: Int = 90,
        completion: @escaping ([String: Any]?) -> Void
    ) {
        self.pickerType = source
        NSLog("[ImagePicker] >>> pickImage called, source=%@", source)
        NSLog("[ImagePicker] >>> About to dispatch to main queue")
        
        // Prevent multiple concurrent picker presentations
        if isPresentingPicker {
            NSLog("[ImagePicker] >>> Already presenting a picker, rejecting new request")
            completion(["error": "Picker already being presented"])
            return
        }
        
        DispatchQueue.main.async {
            NSLog("[ImagePicker] >>> Inside main.async block")
            guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                  let window = windowScene.windows.first,
                  let rootViewController = window.rootViewController else {
                NSLog("[ImagePicker] ERROR: No rootViewController")
                completion(nil)
                return
            }
            
            NSLog("[ImagePicker] >>> Got rootViewController: %@", String(describing: type(of: rootViewController)))
            self.presentingViewController = rootViewController
            
            // Check if there's already a presented view controller that's a picker
            if rootViewController.presentedViewController is UIImagePickerController {
                NSLog("[ImagePicker] >>> Already presenting a picker, rejecting new request")
                completion(["error": "Picker already being presented"])
                return
            }
            
            self.isPresentingPicker = true
            
            switch source {
            case "camera":
                self.presentCamera(maxWidth: maxWidth, maxHeight: maxHeight, quality: quality, completion: completion)
            case "file":
                self.presentDocumentPicker(completion: completion)
            default:
                self.presentPhotoPicker(maxWidth: maxWidth, maxHeight: maxHeight, quality: quality, completion: completion)
            }
        }
        NSLog("[ImagePicker] >>> pickImage method returning (async dispatched)")
    }
    
    private func presentPhotoPicker(maxWidth: Int, maxHeight: Int, quality: Int, completion: @escaping ([String: Any]?) -> Void) {
        // Use UIImagePickerController for auto-dismiss on single selection
        // PHPickerViewController requires manual "Done" tap even for single selection
        guard UIImagePickerController.isSourceTypeAvailable(.photoLibrary) else {
            log("ERROR: Photo library not available")
            completion(["error": "Photo library not available"])
            return
        }
        
        log("Presenting photo library picker")
        let picker = UIImagePickerController()
        picker.sourceType = .photoLibrary
        picker.mediaTypes = [UTType.image.identifier]
        picker.allowsEditing = false
        picker.delegate = self
        
        objc_setAssociatedObject(picker, &ImagePicker.completionKey, completion, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        objc_setAssociatedObject(picker, &ImagePicker.maxWidthKey, maxWidth, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        objc_setAssociatedObject(picker, &ImagePicker.maxHeightKey, maxHeight, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        objc_setAssociatedObject(picker, &ImagePicker.qualityKey, quality, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        objc_setAssociatedObject(picker, &ImagePicker.isPhotoLibraryKey, true, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        
        presentingViewController?.present(picker, animated: true) {
            self.log("Photo library picker presented")
        }
    }
    
    private func presentCamera(maxWidth: Int, maxHeight: Int, quality: Int, completion: @escaping ([String: Any]?) -> Void) {
        guard UIImagePickerController.isSourceTypeAvailable(.camera) else {
            completion(["error": "Camera not available"])
            return
        }
        
        let picker = UIImagePickerController()
        picker.sourceType = .camera
        picker.mediaTypes = [UTType.image.identifier]
        picker.allowsEditing = false
        picker.delegate = self
        
        objc_setAssociatedObject(picker, &ImagePicker.completionKey, completion, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        objc_setAssociatedObject(picker, &ImagePicker.maxWidthKey, maxWidth, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        objc_setAssociatedObject(picker, &ImagePicker.maxHeightKey, maxHeight, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        objc_setAssociatedObject(picker, &ImagePicker.qualityKey, quality, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        
        presentingViewController?.present(picker, animated: true)
    }
    
    private func presentDocumentPicker(completion: @escaping ([String: Any]?) -> Void) {
        let supportedTypes: [UTType] = [.image, .jpeg, .png, .gif, .webP, .heic]
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: supportedTypes)
        picker.allowsMultipleSelection = false
        picker.delegate = self
        
        objc_setAssociatedObject(picker, &ImagePicker.completionKey, completion, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
        
        presentingViewController?.present(picker, animated: true)
    }
    
    private func processImage(_ image: UIImage, maxWidth: Int, maxHeight: Int, quality: Int, completion: @escaping ([String: Any]?) -> Void) {
        // Resize if needed
        let resizedImage = resizeImage(image, maxWidth: CGFloat(maxWidth), maxHeight: CGFloat(maxHeight))
        log("Resized image to: \(resizedImage.size)")
        
        // Save to temporary file
        guard let data = resizedImage.jpegData(compressionQuality: CGFloat(quality) / 100.0) else {
            log("ERROR: Failed to encode image as JPEG")
            completion(["error": "Failed to encode image"])
            return
        }
        
        let tempDir = FileManager.default.temporaryDirectory
        let fileName = "picked_\(UUID().uuidString).jpg"
        let fileURL = tempDir.appendingPathComponent(fileName)
        log("Saving to: \(fileURL.path)")
        
        do {
            try data.write(to: fileURL)
            log("File saved successfully, size: \(data.count) bytes")
            let result: [String: Any] = [
                "path": fileURL.path,
                "mimeType": "image/jpeg",
                "width": Int(resizedImage.size.width * resizedImage.scale),
                "height": Int(resizedImage.size.height * resizedImage.scale)
            ]
            completion(result)
        } catch {
            log("ERROR: Failed to save image: \(error.localizedDescription)")
            completion(["error": "Failed to save image: \(error.localizedDescription)"])
        }
    }
    
    private func resizeImage(_ image: UIImage, maxWidth: CGFloat, maxHeight: CGFloat) -> UIImage {
        let size = image.size
        let widthRatio = maxWidth / size.width
        let heightRatio = maxHeight / size.height
        let ratio = min(widthRatio, heightRatio, 1.0) // Don't upscale
        
        if ratio >= 1.0 {
            return image
        }
        
        let newSize = CGSize(width: size.width * ratio, height: size.height * ratio)
        
        UIGraphicsBeginImageContextWithOptions(newSize, false, image.scale)
        image.draw(in: CGRect(origin: .zero, size: newSize))
        let resizedImage = UIGraphicsGetImageFromCurrentImageContext()
        UIGraphicsEndImageContext()
        
        return resizedImage ?? image
    }
}

extension ImagePicker: PHPickerViewControllerDelegate {
    public func picker(_ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]) {
        picker.dismiss(animated: true)
        
        guard let completion = objc_getAssociatedObject(picker, &ImagePicker.completionKey) as? ([String: Any]?) -> Void,
              let maxWidth = objc_getAssociatedObject(picker, &ImagePicker.maxWidthKey) as? Int,
              let maxHeight = objc_getAssociatedObject(picker, &ImagePicker.maxHeightKey) as? Int,
              let quality = objc_getAssociatedObject(picker, &ImagePicker.qualityKey) as? Int,
              let result = results.first else {
            return
        }
        
        let itemProvider = result.itemProvider
        if itemProvider.canLoadObject(ofClass: UIImage.self) {
            itemProvider.loadObject(ofClass: UIImage.self) { [weak self] image, error in
                DispatchQueue.main.async {
                    if let error = error {
                        completion(["error": error.localizedDescription])
                        return
                    }
                    
                    guard let image = image as? UIImage else {
                        completion(["error": "Failed to load image"])
                        return
                    }
                    
                    self?.processImage(image, maxWidth: maxWidth, maxHeight: maxHeight, quality: quality, completion: completion)
                }
            }
        } else {
            completion(["error": "Cannot load image from provider"])
        }
    }
}

extension ImagePicker: UIImagePickerControllerDelegate, UINavigationControllerDelegate {
    public func imagePickerController(_ picker: UIImagePickerController, didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
        log("imagePickerController didFinishPickingMediaWithInfo")
        isPresentingPicker = false
        picker.dismiss(animated: true)
        
        let completion = objc_getAssociatedObject(picker, &ImagePicker.completionKey) as? ([String: Any]?) -> Void
        let maxWidth = objc_getAssociatedObject(picker, &ImagePicker.maxWidthKey) as? Int
        let maxHeight = objc_getAssociatedObject(picker, &ImagePicker.maxHeightKey) as? Int
        let quality = objc_getAssociatedObject(picker, &ImagePicker.qualityKey) as? Int
        let image = info[.originalImage] as? UIImage
        
        log("Debug: completion=\(completion != nil), maxWidth=\(maxWidth != nil), maxHeight=\(maxHeight != nil), quality=\(quality != nil), image=\(image != nil)")
        
        guard let completion = completion,
              let maxWidth = maxWidth,
              let maxHeight = maxHeight,
              let quality = quality,
              let image = image else {
            log("ERROR: Missing data in didFinishPickingMediaWithInfo")
            return
        }
        
        log("Got image: \(image.size), scale: \(image.scale)")
        processImage(image, maxWidth: maxWidth, maxHeight: maxHeight, quality: quality, completion: completion)
    }
    
    public func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
        log("imagePickerControllerDidCancel")
        isPresentingPicker = false
        picker.dismiss(animated: true)
        guard let completion = objc_getAssociatedObject(picker, &ImagePicker.completionKey) as? ([String: Any]?) -> Void else {
            return
        }
        log("Calling completion with nil (cancelled)")
        completion(nil)
    }
}

// PHPickerViewControllerDelegate kept for potential future use (e.g., multi-select)
// Currently using UIImagePickerController for gallery for auto-dismiss behavior

extension ImagePicker: UIDocumentPickerDelegate {
    public func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
        controller.dismiss(animated: true)
        isPresentingPicker = false
        
        guard let completion = objc_getAssociatedObject(controller, &ImagePicker.completionKey) as? ([String: Any]?) -> Void,
              let url = urls.first else {
            return
        }
        
        // Start accessing security-scoped resource
        let shouldStopAccessing = url.startAccessingSecurityScopedResource()
        defer {
            if shouldStopAccessing {
                url.stopAccessingSecurityScopedResource()
            }
        }
        
        do {
            let data = try Data(contentsOf: url)
            guard let image = UIImage(data: data) else {
                completion(["error": "Failed to load image from file"])
                return
            }
            
            // Get file extension for mime type
            let pathExtension = url.pathExtension.lowercased()
            let mimeType: String
            switch pathExtension {
            case "jpg", "jpeg": mimeType = "image/jpeg"
            case "png": mimeType = "image/png"
            case "gif": mimeType = "image/gif"
            case "webp": mimeType = "image/webp"
            case "heic": mimeType = "image/heic"
            default: mimeType = "application/octet-stream"
            }
            
            let result: [String: Any] = [
                "path": url.path,
                "mimeType": mimeType,
                "width": Int(image.size.width * image.scale),
                "height": Int(image.size.height * image.scale)
            ]
            completion(result)
        } catch {
            completion(["error": "Failed to read file: \(error.localizedDescription)"])
        }
    }
    
    public func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
        controller.dismiss(animated: true)
        isPresentingPicker = false
        guard let completion = objc_getAssociatedObject(controller, &ImagePicker.completionKey) as? ([String: Any]?) -> Void else {
            return
        }
        completion(nil)
    }
}

// C interface for Rust
@_cdecl("scorpion_pick_image")
public func scorpionPickImage(
    source: UnsafePointer<CChar>?,
    maxWidth: Int32,
    maxHeight: Int32,
    quality: Int32,
    callback: @escaping @convention(c) (UnsafePointer<CChar>?) -> Void
) {
    let sourceStr = source.map { String(cString: $0) } ?? "gallery"
    
    // Create ImagePicker and retain it until the picker is dismissed
    // We use a static dictionary to store the picker instances
    let picker = ImagePicker()
    
    // Store the picker in a static dictionary to prevent deallocation
    // Use the callback pointer as the key (unique per call)
    let key = Unmanaged.passUnretained(callback as AnyObject).toOpaque()
    ImagePicker.retainedPickers[key] = picker
    
    NSLog("[ImagePicker] C function scorpionPickImage called, source=%@, key=%@", sourceStr, String(describing: key))
    
    picker.log("scorpionPickImage called, source=\(sourceStr), key=\(key)")
    
    picker.pickImage(
        source: sourceStr,
        maxWidth: Int(maxWidth),
        maxHeight: Int(maxHeight),
        quality: Int(quality)
    ) { result in
        picker.log("pickImage callback, result=\(result != nil ? "success" : "nil/cancelled")")
        
        var jsonString: String
        if let result = result {
            if let error = result["error"] as? String {
                jsonString = "{\"error\": \"\(error)\"}"
            } else if let path = result["path"] as? String,
                      let mimeType = result["mimeType"] as? String,
                      let width = result["width"] as? Int,
                      let height = result["height"] as? Int {
                picker.log("Returning path: \(path)")
                jsonString = "{\"path\": \"\(path)\", \"mimeType\": \"\(mimeType)\", \"width\": \(width), \"height\": \(height)}"
            } else {
                jsonString = "{\"error\": \"Unknown result format\"}"
            }
        } else {
            // User cancelled - return special cancelled marker
            jsonString = "{\"cancelled\": true}"
        }
        
        picker.log("About to call Rust callback with: \(jsonString)")
        let cString = strdup(jsonString)
        callback(cString)
        free(cString)
        
        // Clean up the retained picker
        ImagePicker.retainedPickers[key] = nil
        picker.log("Cleaned up picker for key=\(key)")
    }
}
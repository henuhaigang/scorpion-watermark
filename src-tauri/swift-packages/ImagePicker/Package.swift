// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ImagePicker",
    platforms: [
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "ImagePicker",
            type: .static,
            targets: ["ImagePicker"]
        ),
    ],
    targets: [
        .target(
            name: "ImagePicker",
            path: "Sources/ImagePicker"
        ),
    ]
)
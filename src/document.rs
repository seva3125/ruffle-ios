use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send};
use objc2_foundation::{NSObjectProtocol, NSURL};
use objc2_ui_kit::UIDocument;

// There is no standardized UTI for SWFs, so this is one we picked.
pub const SWF_UTI: &str = "com.adobe.swf";

// Temporary until we publish the package
pub const RUF_UTI: &str = "com.example.rs.ruffle.bundle";

#[derive(Default, Debug)]
pub struct Ivars {}

define_class!(
    #[unsafe(super(UIDocument))]
    #[name = "SWFDocument"]
    #[ivars = Ivars]
    #[derive(Debug)]
    pub struct SWFDocument;

    unsafe impl NSObjectProtocol for SWFDocument {}

    /// UIDocument override.
    impl SWFDocument {
        #[unsafe(method_id(initWithFileURL:))]
        fn _init_with_file_url(this: Allocated<Self>, url: &NSURL) -> Retained<Self> {
            tracing::info!("SWFDocument init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send![super(this), initWithFileURL: url] }
        }
    }
);

define_class!(
    #[unsafe(super(UIDocument))]
    #[name = "BundleDocument"]
    #[ivars = Ivars]
    #[derive(Debug)]
    pub struct BundleDocument;

    unsafe impl NSObjectProtocol for BundleDocument {}

    /// UIDocument override.
    impl BundleDocument {
        #[unsafe(method_id(initWithFileURL:))]
        fn _init_with_file_url(this: Allocated<Self>, url: &NSURL) -> Retained<Self> {
            tracing::info!("BundleDocument init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send![super(this), initWithFileURL: url] }
        }
    }
);

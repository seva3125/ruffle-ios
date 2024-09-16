use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{NSObjectProtocol, NSURL};
use objc2_ui_kit::UIDocument;

// There is no standardized UTI for SWFs, so this is one we picked.
pub const SWF_UTI: &str = "com.adobe.swf";

// Temporary until we publish the package
pub const RUF_UTI: &str = "com.example.rs.ruffle.bundle";

#[derive(Default)]
pub struct Ivars {}

declare_class!(
    #[derive(Debug)]
    pub struct SWFDocument;

    unsafe impl ClassType for SWFDocument {
        type Super = UIDocument;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "SWFDocument";
    }

    impl DeclaredClass for SWFDocument {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for SWFDocument {}

    unsafe impl SWFDocument {
        #[method_id(initWithFileURL:)]
        fn _init_with_file_url(this: Allocated<Self>, url: &NSURL) -> Retained<Self> {
            tracing::info!("document init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithFileURL: url] }
        }
    }
);

declare_class!(
    #[derive(Debug)]
    pub struct BundleDocument;

    unsafe impl ClassType for BundleDocument {
        type Super = UIDocument;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "BundleDocument";
    }

    impl DeclaredClass for BundleDocument {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for BundleDocument {}

    unsafe impl BundleDocument {
        #[method_id(initWithFileURL:)]
        fn _init_with_file_url(this: Allocated<Self>, url: &NSURL) -> Retained<Self> {
            tracing::info!("document init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithFileURL: url] }
        }
    }
);

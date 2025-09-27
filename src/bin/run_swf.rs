//! Run an SWF without setting up navigation, a data model and everything.
use std::cell::OnceCell;

use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send, ClassType, DefinedClass as _, MainThreadOnly};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use objc2_ui_kit::{UIApplication, UIApplicationDelegate, UIScreen, UIWindow};

use ruffle_frontend_utils::content::PlayingContent;
use ruffle_frontend_utils::player_options::PlayerOptions;
use ruffle_ios::{init_logging, launch, PlayerController};
use url::Url;

#[derive(Debug)]
pub struct Ivars {
    window: OnceCell<Retained<UIWindow>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "AppDelegate"]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    #[derive(Debug)]
    pub struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    /// Called by UIKitApplicationMain.
    impl AppDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(Ivars {
                window: OnceCell::new(),
            });
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl UIApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, _application: &UIApplication) {
            tracing::info!("applicationDidFinishLaunching:");
            self.setup();
        }
    }
);

impl AppDelegate {
    fn setup(&self) {
        let movie_path = std::env::args_os().skip(1).next();
        let movie_path = movie_path.expect("must provide a path or URL to an SWF to run");
        let mtm = MainThreadMarker::from(self);

        #[allow(deprecated)] // Unsure how else we should do this when setting up?
        let frame = UIScreen::mainScreen(mtm).bounds();

        #[allow(deprecated)]
        let window = UIWindow::initWithFrame(mtm.alloc(), frame);

        let movie_path = std::path::absolute(movie_path).unwrap();
        let content = PlayingContent::DirectFile(Url::from_file_path(movie_path).unwrap());

        let view_controller = PlayerController::new(mtm, content, PlayerOptions::default());
        window.setRootViewController(Some(&view_controller));

        window.makeKeyAndVisible();

        self.ivars()
            .window
            .set(window)
            .expect("can only initialize once");
    }
}

#[tokio::main]
async fn main() {
    init_logging();
    launch(None, Some(AppDelegate::class()));
}

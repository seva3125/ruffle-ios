use objc2::rc::{Allocated, Retained};
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, DefinedClass, MainThreadOnly, Message};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSDictionary, NSObject, NSObjectProtocol, NSSet,
};
use objc2_ui_kit::{
    UIApplication, UIApplicationDelegate, UIApplicationLaunchOptionsKey, UISceneConfiguration,
    UISceneConnectionOptions, UISceneSession, UIWindow,
};

use crate::storage;

pub struct Ivars {
    window: std::cell::Cell<Option<Retained<UIWindow>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "AppDelegate"]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    pub struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    /// NSObject.
    impl AppDelegate {
        // Called by UIKitApplicationMain
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(Ivars {
                window: std::cell::Cell::new(None),
            });
            unsafe { msg_send![super(this), init] }
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UIApplicationDelegate for AppDelegate {
        // NOTE: Probably only called by storyboards?
        #[unsafe(method_id(window))]
        fn window(&self) -> Option<Retained<UIWindow>> {
            let window = self.ivars().window.take();
            self.ivars().window.set(window.clone());
            window
        }

        #[unsafe(method(setWindow:))]
        fn setWindow(&self, window: Option<&UIWindow>) {
            self.ivars().window.set(window.map(|w| w.retain()));
        }

        #[unsafe(method(application:didFinishLaunchingWithOptions:))]
        fn didFinishLaunching(
            &self,
            _application: &UIApplication,
            _launch_options: Option<&NSDictionary<UIApplicationLaunchOptionsKey, AnyObject>>,
        ) -> bool {
            tracing::info!("applicationDidFinishLaunching:");
            storage::setup();
            true
        }

        #[unsafe(method(applicationWillEnterForeground:))]
        fn applicationWillEnterForeground(&self, _application: &UIApplication) {
            tracing::info!("applicationWillEnterForeground:");
        }

        #[unsafe(method(applicationDidBecomeActive:))]
        fn applicationDidBecomeActive(&self, _application: &UIApplication) {
            tracing::info!("applicationDidBecomeActive:");
        }

        #[unsafe(method(applicationWillResignActive:))]
        fn applicationWillResignActive(&self, _application: &UIApplication) {
            tracing::info!("applicationWillResignActive:");
        }

        #[unsafe(method(applicationDidEnterBackground:))]
        fn applicationDidEnterBackground(&self, _application: &UIApplication) {
            tracing::info!("applicationDidEnterBackground:");
        }

        #[unsafe(method_id(application:configurationForConnectingSceneSession:options:))]
        fn _application_configuration_for_connecting_scene_session_options(
            &self,
            _application: &UIApplication,
            connecting_scene_session: &UISceneSession,
            _options: &UISceneConnectionOptions,
        ) -> Retained<UISceneConfiguration> {
            tracing::info!("application:configurationForConnectingSceneSession:options:");
            // Called when a new scene session is being created.
            // Use this method to select a configuration to create the new scene with.
            let mtm = MainThreadMarker::from(self);
            unsafe {
                UISceneConfiguration::initWithName_sessionRole(
                    mtm.alloc(),
                    Some(ns_string!("Default Configuration")),
                    &connecting_scene_session.role(),
                )
            }
        }

        #[unsafe(method(application:didDiscardSceneSessions:))]
        fn _application_did_discard_scene_sessions(
            &self,
            _application: &UIApplication,
            _scene_sessions: &NSSet<UISceneSession>,
        ) {
            tracing::info!("application:didDiscardSceneSessions:");
            // Called when the user discards a scene session.
            // If any sessions were discarded while the application was not running, this will be called shortly after application:didFinishLaunchingWithOptions.
            // Use this method to release any resources that were specific to the discarded scenes, as they will not return.
        }
    }
);

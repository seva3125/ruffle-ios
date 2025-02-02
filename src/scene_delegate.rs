use std::cell::Cell;

use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send, DefinedClass as _, MainThreadOnly, Message};
use objc2_foundation::{NSObjectProtocol, NSSet, NSURL};
use objc2_ui_kit::{
    UINavigationController, UIOpenURLContext, UIResponder, UIScene, UISceneConnectionOptions,
    UISceneDelegate, UISceneSession, UIWindow, UIWindowScene, UIWindowSceneDelegate,
};

use crate::{storage, PlayerController};

pub struct Ivars {
    window: Cell<Option<Retained<UIWindow>>>,
}

define_class!(
    #[unsafe(super(UIResponder))]
    #[name = "SceneDelegate"]
    #[ivars = Ivars]
    pub struct SceneDelegate;

    /// Called by UIStoryboard
    impl SceneDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            tracing::info!("init scene");
            let this = this.set_ivars(Ivars {
                window: Cell::new(None),
            });
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for SceneDelegate {}

    #[allow(non_snake_case)]
    unsafe impl UISceneDelegate for SceneDelegate {
        #[unsafe(method(scene:willConnectToSession:options:))]
        fn scene_willConnectToSession_options(
            &self,
            _scene: &UIScene,
            _session: &UISceneSession,
            _connection_options: &UISceneConnectionOptions,
        ) {
            tracing::info!("scene:willConnectToSession:options:");
            // Use this method to optionally configure and attach the UIWindow `window` to the provided UIWindowScene `scene`.
            // If using a storyboard, the `window` property will automatically be initialized and attached to the scene.
            // This delegate does not imply the connecting scene or session are new (see `application:configurationForConnectingSceneSession` instead).
        }

        #[unsafe(method(sceneDidDisconnect:))]
        fn sceneDidDisconnect(&self, _scene: &UIScene) {
            tracing::info!("sceneDidDisconnect:");
            // Called as the scene is being released by the system.
            // This occurs shortly after the scene enters the background, or when its session is discarded.
            // Release any resources associated with this scene that can be re-created the next time the scene connects.
            // The scene may re-connect later, as its session was not necessarily discarded (see `application:didDiscardSceneSessions` instead).
        }

        #[unsafe(method(sceneDidBecomeActive:))]
        fn sceneDidBecomeActive(&self, scene: &UIScene) {
            tracing::info!("sceneDidBecomeActive:");

            // Restart playing.
            let nav = get_navigation_controller(scene);
            for controller in unsafe { nav.viewControllers() } {
                if let Some(controller) = controller.downcast_ref::<PlayerController>() {
                    controller.view().start();
                }
            }
        }

        #[unsafe(method(sceneWillResignActive:))]
        fn sceneWillResignActive(&self, scene: &UIScene) {
            tracing::info!("sceneWillResignActive:");

            // Stop playing.
            // TODO: Is this the best place to do this?
            let nav = get_navigation_controller(scene);
            for controller in unsafe { nav.viewControllers() } {
                if let Some(controller) = controller.downcast_ref::<PlayerController>() {
                    controller.view().stop();
                }
            }
        }

        #[unsafe(method(sceneWillEnterForeground:))]
        fn sceneWillEnterForeground(&self, _scene: &UIScene) {
            tracing::info!("sceneWillEnterForegrounds:");
        }

        #[unsafe(method(sceneDidEnterBackground:))]
        fn sceneDidEnterBackground(&self, scene: &UIScene) {
            tracing::info!("sceneDidEnterBackground:");

            // Flush when going to the background.
            let nav = get_navigation_controller(scene);
            for controller in unsafe { nav.viewControllers() } {
                if let Some(controller) = controller.downcast_ref::<PlayerController>() {
                    controller.view().flush();
                }
            }
        }

        #[unsafe(method(scene:openURLContexts:))]
        fn scene_openURLContexts(&self, scene: &UIScene, url_contexts: &NSSet<UIOpenURLContext>) {
            tracing::info!(?url_contexts, "scene:openURLContexts:");

            for context in url_contexts {
                let url = unsafe { context.URL() };

                // TODO: Do something else when this is set?
                let _ = unsafe { context.options().openInPlace() };

                if storage::movie_from_url(&url).is_none() {
                    storage::add_movie(&url);
                } else {
                    // This is intentional, when the user opens URLs from outside
                    // the app, we only want to add them to the library if not
                    // already there.
                    tracing::debug!("did not add existing movie {url:?}");
                }
            }

            if url_contexts.count() == 1 {
                let context = url_contexts.anyObject().unwrap();
                let url = unsafe { context.URL() };
                // Start playing this one immediately
                play_url(scene, &url);
            }
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UIWindowSceneDelegate for SceneDelegate {
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
    }
);

impl Drop for SceneDelegate {
    fn drop(&mut self) {
        tracing::info!("drop scene");
    }
}

fn get_navigation_controller(scene: &UIScene) -> Retained<UINavigationController> {
    let scene = scene.downcast_ref::<UIWindowScene>().unwrap();
    // FIXME: Assumes single-window
    let window = unsafe { scene.windows() }.firstObject().unwrap();
    let root = window.rootViewController().unwrap();
    root.downcast::<UINavigationController>().unwrap()
}

fn play_url(scene: &UIScene, url: &NSURL) -> Option<()> {
    let _span = tracing::info_span!("play_url").entered();

    let nav = get_navigation_controller(scene);

    // TODO: Investigate if we really want to do this?
    unsafe { nav.popToRootViewControllerAnimated(true) };

    let movie = storage::movie_from_url(url).expect("we just added the movie");
    let player_controller = PlayerController::empty(scene.mtm());
    player_controller.setup_movie(&movie);
    unsafe { nav.pushViewController_animated(&player_controller, true) };

    Some(())
}

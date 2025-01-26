use std::cell::OnceCell;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fmt, io, ptr};

use objc2::rc::{Allocated, Retained};
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, AllocAnyThread, DefinedClass as _};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSBundle, NSCoder, NSObjectProtocol, NSString,
};
use objc2_ui_kit::{NSDataAsset, UIViewController};
use ruffle_core::config::Letterbox;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder};
use ruffle_frontend_utils::backends::audio::CpalAudioBackend;
use ruffle_frontend_utils::backends::executor::{AsyncExecutor, PollRequester};
use ruffle_frontend_utils::backends::navigator::{ExternalNavigatorBackend, NavigatorInterface};
use ruffle_frontend_utils::content::PlayingContent;
use url::Url;

use crate::player_view::PlayerView;

#[derive(Clone)]
pub struct EventSender(Rc<OnceCell<Arc<AsyncExecutor<EventSender>>>>);

impl PollRequester for EventSender {
    fn request_poll(&self) {
        eprintln!("request_poll, main: {}", MainThreadMarker::new().is_some());
        self.0.get().expect("initialized").poll_all();
    }
}

#[derive(Default)]
pub struct Ivars {
    movie_path: Option<String>,
    player: OnceCell<Arc<Mutex<Player>>>,
    executor: OnceCell<Arc<AsyncExecutor<EventSender>>>,
}

impl fmt::Debug for Ivars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ivars")
            .field("movie_path", &self.movie_path)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
struct Navigator;

impl NavigatorInterface for Navigator {
    fn navigate_to_website(&self, _url: Url, _ask: bool) {}

    fn open_file(&self, path: &Path) -> io::Result<File> {
        File::open(path)
    }

    async fn confirm_socket(&self, _host: &str, _port: u16) -> bool {
        true
    }
}

define_class!(
    #[unsafe(super(UIViewController))]
    #[name = "PlayerController"]
    #[ivars = Ivars]
    #[derive(Debug)]
    pub struct PlayerController;

    unsafe impl NSObjectProtocol for PlayerController {}

    /// UIViewController.
    impl PlayerController {
        #[unsafe(method_id(initWithNibName:bundle:))]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("player controller init");
            let this = this.set_ivars(Ivars::default());
            unsafe {
                msg_send![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil]
            }
        }

        #[unsafe(method_id(initWithCoder:))]
        fn _init_with_coder(this: Allocated<Self>, coder: &NSCoder) -> Option<Retained<Self>> {
            tracing::info!("player controller init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send![super(this), initWithCoder: coder] }
        }

        #[unsafe(method(loadView))]
        fn _load_view(&self) {
            self.load_view();
        }

        #[unsafe(method(viewDidLoad))]
        fn _view_did_load(&self) {
            // Xcode template calls super at the beginning
            let _: () = unsafe { msg_send![super(self), viewDidLoad] };
            self.view_did_load();
        }

        #[unsafe(method(viewIsAppearing:))]
        fn _view_is_appearing(&self, animated: bool) {
            self.view_is_appearing(animated);
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewIsAppearing: animated] };
        }

        #[unsafe(method(viewWillDisappear:))]
        fn _view_will_disappear(&self, animated: bool) {
            self.view_will_disappear(animated);
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewWillDisappear: animated] };
        }

        #[unsafe(method(viewDidDisappear:))]
        fn _view_did_disappear(&self, animated: bool) {
            self.view_did_disappear(animated);
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewDidDisappear: animated] };
        }
    }

    /// UIResponder
    #[allow(non_snake_case)]
    impl PlayerController {
        #[unsafe(method(canBecomeFirstResponder))]
        fn canBecomeFirstResponder(&self) -> bool {
            true
        }

        #[unsafe(method(becomeFirstResponder))]
        fn becomeFirstResponder(&self) -> bool {
            tracing::info!("player controller becomeFirstResponder");
            unsafe { self.view().becomeFirstResponder() };
            true
        }

        #[unsafe(method(canResignFirstResponder))]
        fn canResignFirstResponder(&self) -> bool {
            true
        }

        #[unsafe(method(resignFirstResponder))]
        fn resignFirstResponder(&self) -> bool {
            tracing::info!("controller resignFirstResponder");
            true
        }
    }
);

impl PlayerController {
    pub fn new(mtm: MainThreadMarker, movie_path: String) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(Ivars {
            movie_path: Some(movie_path),
            player: OnceCell::new(),
            executor: OnceCell::new(),
        });
        let nil = ptr::null::<AnyObject>();
        unsafe { msg_send![super(this), initWithNibName: nil, bundle: nil] }
    }

    fn load_view(&self) {
        tracing::info!("player loadView");
        let mtm = MainThreadMarker::from(self);
        let view = PlayerView::initWithFrame(
            mtm.alloc(),
            CGRect::new(CGPoint::ZERO, CGSize::new(1.0, 1.0)),
        );
        self.setView(Some(&view));
    }

    fn view_did_load(&self) {
        tracing::info!("player viewDidLoad");

        // TODO: Specify safe area somehow
        let view = self.view();
        let renderer = view.create_renderer();

        let sender = EventSender(Rc::new(OnceCell::new()));
        let (executor, future_spawner) = AsyncExecutor::new(sender.clone());
        sender
            .0
            .set(executor.clone())
            .unwrap_or_else(|_| panic!("init once"));

        let movie_url = Url::parse("file://movie.swf").unwrap();
        let navigator = ExternalNavigatorBackend::new(
            movie_url.clone(),
            None,
            None,
            future_spawner,
            None,
            true,
            ruffle_core::backend::navigator::OpenURLMode::Allow,
            Default::default(),
            ruffle_core::backend::navigator::SocketMode::Allow,
            Rc::new(PlayingContent::DirectFile(movie_url)),
            Navigator,
        );

        let mut builder = PlayerBuilder::new()
            .with_renderer(renderer)
            .with_navigator(navigator);

        // Temporary until we figure out actual loading
        let movie = if let Some(path) = self.ivars().movie_path.as_deref() {
            SwfMovie::from_path(path, None).expect("failed loading movie")
        } else {
            let asset =
                unsafe { NSDataAsset::initWithName(NSDataAsset::alloc(), ns_string!("logo-anim")) }
                    .expect("asset store should contain logo-anim");
            let data = unsafe { asset.data() };
            // SAFETY: SwfMovie::from_data won't modify the NSData.
            let bytes = unsafe { data.as_bytes_unchecked() };
            SwfMovie::from_data(bytes, "file://logo-anim.swf".into(), None).expect("loading movie")
        };
        builder = builder.with_movie(movie);

        match CpalAudioBackend::new(None) {
            Ok(audio) => builder = builder.with_audio(audio),
            Err(e) => tracing::error!("Unable to create audio device: {e}"),
        }

        let player = builder.build();

        let mut player_lock = player.lock().unwrap();
        // player_lock.fetch_root_movie(
        //     self.ivars().movie_url.clone(),
        //     vec![],
        //     Box::new(|metadata| {
        //         eprintln!("got movie: {:?}", metadata);
        //     }),
        // );
        player_lock.set_letterbox(Letterbox::On);
        drop(player_lock);

        view.set_player(player.clone());
        self.ivars()
            .player
            .set(player)
            .unwrap_or_else(|_| panic!("viewDidLoad once"));
        self.ivars()
            .executor
            .set(executor)
            .unwrap_or_else(|_| panic!("viewDidLoad once"));
    }

    fn view_is_appearing(&self, _animated: bool) {
        tracing::info!("player viewIsAppearing:");

        self.view().start();
    }

    fn view_will_disappear(&self, _animated: bool) {
        tracing::info!("player viewWillDisappear:");

        self.view().stop();
    }

    fn view_did_disappear(&self, _animated: bool) {
        tracing::info!("player viewDidDisappear:");

        self.player_lock().flush_shared_objects();
    }

    fn view(&self) -> Retained<PlayerView> {
        let view = (**self).view().expect("controller loads view");
        view.downcast().expect("must have correct view type")
    }

    #[track_caller]
    fn player_lock(&self) -> MutexGuard<'_, Player> {
        self.ivars()
            .player
            .get()
            .expect("player initialized")
            .lock()
            .expect("player lock")
    }
}

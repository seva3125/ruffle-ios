use objc2::runtime::AnyClass;
use objc2::ClassType;
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_ui_kit::UIApplication;

mod add_controller;
mod app_delegate;
mod edit_controller;
mod library_controller;
mod player_controller;
mod player_view;
mod scene_delegate;
mod storage;

pub use self::app_delegate::AppDelegate;
pub use self::player_controller::PlayerController;
pub use self::player_view::PlayerView;

pub fn init_logging() {
    // Emit logging to either OSLog or stderr, depending on if using Mac
    // Catalyst or native.
    // TODO: If running Mac Catalyst under Xcode
    let filter = log::LevelFilter::Info;
    if cfg!(target_abi = "macabi") {
        simple_logger::SimpleLogger::new()
            .with_level(filter)
            .env()
            .init()
            .unwrap();
    } else {
        oslog::OsLogger::new(module_path!())
            .level_filter(filter)
            .init()
            .unwrap();
    }
}

pub fn launch(app_class: Option<&AnyClass>, delegate_class: Option<&AnyClass>) {
    // Set inside Info.plist
    let _ = scene_delegate::SceneDelegate::class();

    // These classes are loaded from a storyboard,
    // and hence need to be initialized first.
    // See also [storyboard_connections.h]
    let _ = player_view::PlayerView::class();
    let _ = player_controller::PlayerController::class();
    let _ = library_controller::LibraryController::class();
    let _ = edit_controller::EditController::class();
    let _ = add_controller::AddController::class();

    // This is loaded by CoreData
    let _ = storage::Movie::class();
    let _ = storage::MovieData::class();

    let mtm = MainThreadMarker::new().unwrap();
    UIApplication::main(
        app_class.map(|cls| NSString::from_class(cls)).as_deref(),
        delegate_class
            .map(|cls| NSString::from_class(cls))
            .as_deref(),
        mtm,
    );
}

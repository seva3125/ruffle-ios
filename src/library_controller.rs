use std::cell::OnceCell;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, AllocAnyThread, ClassType, DefinedClass as _, Message};
use objc2_core_data::{
    NSFetchedResultsController, NSFetchedResultsControllerDelegate, NSFetchedResultsSectionInfo,
};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSArray, NSBundle, NSCoder, NSIndexPath, NSInteger, NSObject,
    NSObjectProtocol, NSString, NSURL,
};
use objc2_ui_kit::{
    NSDataAsset, UIBarButtonItem, UIDocumentPickerDelegate, UIDocumentPickerViewController,
    UILabel, UITableView, UITableViewCell, UITableViewController, UITableViewDataSource,
};
#[allow(deprecated)]
use objc2_ui_kit::{UIDocumentPickerMode, UIStoryboardSegue};
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::PlayerBuilder;
use ruffle_frontend_utils::backends::audio::CpalAudioBackend;
use ruffle_frontend_utils::bundle::info::BundleInformation;
use ruffle_frontend_utils::player_options::PlayerOptions;
use url::Url;

use crate::document::{RUF_UTI, SWF_UTI};
use crate::edit_controller::EditController;
use crate::storage::Movie;
use crate::{storage, PlayerController, PlayerView};

#[derive(Debug)]
pub struct Ivars {
    logo_view: OnceCell<Retained<PlayerView>>,
    fetched_movies: Retained<NSFetchedResultsController<Movie>>,
}

define_class!(
    #[unsafe(super(UITableViewController))]
    #[name = "LibraryController"]
    #[ivars = Ivars]
    #[derive(Debug)]
    pub struct LibraryController;

    unsafe impl NSObjectProtocol for LibraryController {}

    /// UIViewController.
    impl LibraryController {
        #[unsafe(method_id(initWithNibName:bundle:))]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("library init");
            let this = this.set_ivars(Ivars {
                logo_view: Default::default(),
                fetched_movies: storage::all_movies(),
            });
            let this: Retained<Self> = unsafe {
                msg_send![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil]
            };
            unsafe {
                this.ivars()
                    .fetched_movies
                    .setDelegate(Some(ProtocolObject::from_ref(&*this)))
            };
            this
        }

        #[unsafe(method_id(initWithCoder:))]
        fn _init_with_coder(this: Allocated<Self>, coder: &NSCoder) -> Option<Retained<Self>> {
            tracing::info!("library init");
            let this = this.set_ivars(Ivars {
                logo_view: Default::default(),
                fetched_movies: storage::all_movies(),
            });
            let this: Option<Retained<Self>> =
                unsafe { msg_send![super(this), initWithCoder: coder] };
            if let Some(this) = &this {
                unsafe {
                    this.ivars()
                        .fetched_movies
                        .setDelegate(Some(ProtocolObject::from_ref(&**this)));
                }
            }
            this
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

        #[unsafe(method(prepareForSegue:sender:))]
        #[allow(deprecated)]
        fn _prepare_for_segue(&self, segue: &UIStoryboardSegue, sender: Option<&NSObject>) {
            self.prepare_for_segue(segue, sender.expect("has sender"));
        }
    }

    /// Storyboard
    /// See storyboard_connections.h
    impl LibraryController {
        #[unsafe(method(setLogoView:))]
        fn _set_logo_view(&self, view: Option<&PlayerView>) {
            tracing::trace!("library set logo view");
            let view = view.expect("logo view not null");
            assert!(
                view.isKindOfClass(PlayerView::class()),
                "logo view not a PlayerView"
            );
            self.ivars()
                .logo_view
                .set(view.retain())
                .expect("only set logo view once");
        }

        #[unsafe(method(toggleEditing:))]
        fn _toggle_editing(&self, button: &UIBarButtonItem) {
            tracing::trace!("library toggle editing");
            assert!(
                button.isKindOfClass(UIBarButtonItem::class()),
                "edit button not UIBarButtonItem"
            );
            self.toggle_editing(button);
        }

        #[unsafe(method(cancelEditItem:))]
        #[allow(deprecated)]
        fn _cancel_edit_item(&self, _segue: &UIStoryboardSegue) {}

        #[unsafe(method(saveEditItem:))]
        #[allow(deprecated)]
        fn _save_edit_item(&self, segue: &UIStoryboardSegue) {
            self.save_item(segue);
        }

        #[unsafe(method(showDocumentPicker:))]
        #[allow(deprecated)]
        fn _show_document_picker(&self, _sender: Option<&AnyObject>) {
            self.show_document_picker();
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UITableViewDataSource for LibraryController {
        #[unsafe(method(tableView:numberOfRowsInSection:))]
        fn tableView_numberOfRowsInSection(
            &self,
            _table_view: &UITableView,
            section: NSInteger,
        ) -> NSInteger {
            let sections = unsafe { self.ivars().fetched_movies.sections().unwrap() };
            let section_info = sections.objectAtIndex(section as usize);
            unsafe { section_info.numberOfObjects() as isize }
        }

        #[unsafe(method(numberOfSectionsInTableView:))]
        fn numberOfSectionsInTableView(&self, _table_view: &UITableView) -> NSInteger {
            unsafe { self.ivars().fetched_movies.sections().unwrap().count() as NSInteger }
        }

        #[unsafe(method_id(tableView:cellForRowAtIndexPath:))]
        fn tableView_cellForRowAtIndexPath(
            &self,
            table_view: &UITableView,
            index_path: &NSIndexPath,
        ) -> Retained<UITableViewCell> {
            self.cell_at(table_view, index_path)
        }

        #[unsafe(method_id(tableView:titleForHeaderInSection:))]
        fn tableView_titleForHeaderInSection(
            &self,
            _table_view: &UITableView,
            _section: NSInteger,
        ) -> Option<Retained<NSString>> {
            Some(NSString::from_str("Library"))
        }
    }

    #[allow(non_snake_case)]
    unsafe impl NSFetchedResultsControllerDelegate for LibraryController {}

    #[allow(non_snake_case)]
    unsafe impl UIDocumentPickerDelegate for LibraryController {
        #[unsafe(method(documentPickerWasCancelled:))]
        fn documentPickerWasCancelled(&self, _controller: &UIDocumentPickerViewController) {
            tracing::info!("cancelled document picker");
        }

        #[unsafe(method(documentPicker:didPickDocumentAtURL:))]
        fn documentPicker_didPickDocumentAtURL(
            &self,
            _controller: &UIDocumentPickerViewController,
            url: &NSURL,
        ) {
            tracing::info!("completed document picker: {url:?}");
            storage::add_movie(url);
        }
    }
);

impl LibraryController {
    fn logo_view(&self) -> &PlayerView {
        self.ivars().logo_view.get().expect("logo view initialized")
    }

    fn view_did_load(&self) {
        tracing::info!("library viewDidLoad");

        self.setup_logo();

        unsafe {
            self.ivars()
                .fetched_movies
                .performFetch()
                .expect("failed fetching movies")
        };
    }

    fn setup_logo(&self) {
        let view = self.logo_view();
        let asset =
            unsafe { NSDataAsset::initWithName(NSDataAsset::alloc(), ns_string!("logo-anim")) }
                .expect("asset store should contain logo-anim");
        let data = unsafe { asset.data() };
        // SAFETY: SwfMovie::from_data won't modify the NSData.
        let bytes = unsafe { data.as_bytes_unchecked() };
        let movie =
            SwfMovie::from_data(bytes, "file://logo-anim.swf".into(), None).expect("loading movie");

        let renderer = view.create_renderer();

        let mut builder = PlayerBuilder::new()
            .with_renderer(renderer)
            .with_movie(movie);

        match CpalAudioBackend::new(None) {
            Ok(audio) => builder = builder.with_audio(audio),
            Err(e) => tracing::error!("Unable to create audio device: {e}"),
        }

        view.set_player(builder.build());
        // HACK: Skip first frame to avoid a flicker on startup
        // FIXME: This probably indicates a bug in our timing code?
        view.player_lock().run_frame();
    }

    fn view_is_appearing(&self, _animated: bool) {
        tracing::info!("library viewIsAppearing:");

        self.logo_view().start();
    }

    fn view_will_disappear(&self, _animated: bool) {
        tracing::info!("library viewWillDisappear:");

        self.logo_view().stop();
    }

    fn view_did_disappear(&self, _animated: bool) {
        tracing::info!("library viewDidDisappear:");

        self.logo_view().player_lock().flush_shared_objects();
    }

    #[allow(deprecated)]
    fn prepare_for_segue(&self, segue: &UIStoryboardSegue, sender: &NSObject) {
        let destination = unsafe { segue.destinationViewController() };
        tracing::info!(?destination, "prepareForSegue");

        // Identifiers are set up in the Storyboard
        let identifier = unsafe { segue.identifier() }.expect("segue to have identifier");
        if &*identifier == ns_string!("add-item") {
            // No need to configure AddController
        } else if &*identifier == ns_string!("edit-item") {
            assert!(destination.isKindOfClass(EditController::class()));
            let edit_controller = unsafe { Retained::cast::<EditController>(destination) };
            assert!(sender.isKindOfClass(UITableViewCell::class()));
            let cell = unsafe { &*(sender as *const NSObject as *const UITableViewCell) };

            // TODO
            edit_controller.configure(BundleInformation {
                name: "".into(),
                url: Url::parse("file://").unwrap(),
                player: PlayerOptions::default(),
            });
            dbg!(cell);
        } else if &*identifier == ns_string!("run-item") {
            assert!(destination.isKindOfClass(PlayerController::class()));
            let player_controller = unsafe { Retained::cast::<PlayerController>(destination) };
            assert!(sender.isKindOfClass(UITableViewCell::class()));
            let cell = unsafe { &*(sender as *const NSObject as *const UITableViewCell) };

            // TODO
            dbg!(cell, player_controller);
        } else {
            unreachable!("unknown identifier for segue: {identifier:?}")
        }
    }

    #[allow(deprecated)]
    fn save_item(&self, segue: &UIStoryboardSegue) {
        tracing::info!("saveEditItem");
        let edit_controller = unsafe { segue.sourceViewController() };
        assert!(edit_controller.isKindOfClass(EditController::class()));
        let edit_controller = unsafe { Retained::cast::<EditController>(edit_controller) };
        dbg!(edit_controller); // TODO
    }

    #[allow(deprecated)]
    fn show_document_picker(&self) {
        tracing::info!("show document picker");
        let mtm = MainThreadMarker::from(self);
        let picker = unsafe {
            UIDocumentPickerViewController::initWithDocumentTypes_inMode(
                mtm.alloc(),
                &NSArray::from_slice(&[ns_string!(RUF_UTI), ns_string!(SWF_UTI)]),
                UIDocumentPickerMode::Open,
            )
        };
        unsafe { picker.setDelegate(Some(ProtocolObject::from_ref(self))) };
        // TODO: Consider setting picker.directoryURL to NSDownloadsDirectory,
        // as that's the likely place that people will have their SWFs.

        unsafe { self.presentViewController_animated_completion(&picker, true, None) };
    }

    fn toggle_editing(&self, button: &UIBarButtonItem) {
        unsafe {
            let table_view = self.tableView().expect("has table view");
            let is_editing = !table_view.isEditing();
            table_view.setEditing_animated(is_editing, true);
            button.setTitle(Some(if is_editing {
                ns_string!("Done")
            } else {
                ns_string!("Edit")
            }));
        }
    }

    fn cell_at(
        &self,
        table_view: &UITableView,
        index_path: &NSIndexPath,
    ) -> Retained<UITableViewCell> {
        unsafe {
            let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                ns_string!("library-item"),
                index_path,
            );
            let subviews = cell.contentView().subviews();

            let movie = self.ivars().fetched_movies.objectAtIndexPath(index_path);
            let _url = movie.link();

            // TODO: Cache data here somehow?
            // if url.startAccessingSecurityScopedResource() {
            //     BundleInformation::parse(input);

            //     url.stopAccessingSecurityScopedResource();
            // } else {

            let bundle = BundleInformation {
                name: "Another example".into(),
                url: Url::parse("file:///example2.swf").unwrap(),
                player: PlayerOptions::default(),
            };

            let title = subviews.objectAtIndex(1).downcast::<UILabel>().unwrap();
            title.setText(Some(&NSString::from_str(&bundle.name)));

            let subtitle = subviews.objectAtIndex(2).downcast::<UILabel>().unwrap();
            subtitle.setText(Some(&NSString::from_str(&bundle.url.to_string())));

            cell
        }
    }
}

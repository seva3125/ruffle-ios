use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send};
use objc2_foundation::{NSBundle, NSCoder, NSObjectProtocol, NSString};
use objc2_ui_kit::UIViewController;

#[derive(Default, Debug)]
pub struct Ivars {}

define_class!(
    #[unsafe(super(UIViewController))]
    #[name = "AddController"]
    #[ivars = Ivars]
    #[derive(Debug)]
    pub struct AddController;

    unsafe impl NSObjectProtocol for AddController {}

    /// UIViewController.
    impl AddController {
        #[unsafe(method_id(initWithNibName:bundle:))]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("add init");
            let this = this.set_ivars(Ivars::default());
            unsafe {
                msg_send![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil]
            }
        }

        #[unsafe(method_id(initWithCoder:))]
        fn _init_with_coder(this: Allocated<Self>, coder: &NSCoder) -> Option<Retained<Self>> {
            tracing::info!("add init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send![super(this), initWithCoder: coder] }
        }

        #[unsafe(method(viewDidLoad))]
        fn _view_did_load(&self) {
            // Xcode template calls super at the beginning
            let _: () = unsafe { msg_send![super(self), viewDidLoad] };
            self.view_did_load();
        }

        #[unsafe(method(viewWillAppear:))]
        fn _view_will_appear(&self, animated: bool) {
            self.view_will_appear();
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewWillAppear: animated] };
        }

        #[unsafe(method(viewDidAppear:))]
        fn _view_did_appear(&self, animated: bool) {
            self.view_did_appear();
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewDidAppear: animated] };
        }
    }

    /// Storyboard
    /// See storyboard_connections.h
    impl AddController {
        // #[unsafe(method(setTableView:))]
        // fn _set_table_view(&self, table_view: &UITableView) {
        //     tracing::trace!("edit set table view");
        //     self.ivars()
        //         .table_view
        //         .set(table_view.retain())
        //         .expect("only set table view once");
        // }
    }
);

impl AddController {
    fn view_did_load(&self) {
        tracing::info!("edit viewDidLoad");
    }

    fn view_will_appear(&self) {
        tracing::info!("edit viewWillAppear:");
    }

    fn view_did_appear(&self) {
        tracing::info!("edit viewDidAppear:");
    }
}

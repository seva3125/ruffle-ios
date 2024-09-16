use objc2::rc::{Allocated, Retained};
use objc2::{declare_class, msg_send, msg_send_id, mutability, ClassType, DeclaredClass};
use objc2_foundation::{NSBundle, NSCoder, NSObjectProtocol, NSString};
use objc2_ui_kit::UIViewController;

#[derive(Default)]
pub struct Ivars {}

declare_class!(
    #[derive(Debug)]
    pub struct AddController;

    unsafe impl ClassType for AddController {
        type Super = UIViewController;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "AddController";
    }

    impl DeclaredClass for AddController {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for AddController {}

    unsafe impl AddController {
        #[method_id(initWithNibName:bundle:)]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("add init");
            let this = this.set_ivars(Ivars::default());
            unsafe {
                msg_send_id![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil]
            }
        }

        #[method_id(initWithCoder:)]
        fn _init_with_coder(this: Allocated<Self>, coder: &NSCoder) -> Option<Retained<Self>> {
            tracing::info!("add init");
            let this = this.set_ivars(Ivars::default());
            unsafe { msg_send_id![super(this), initWithCoder: coder] }
        }

        #[method(viewDidLoad)]
        fn _view_did_load(&self) {
            // Xcode template calls super at the beginning
            let _: () = unsafe { msg_send![super(self), viewDidLoad] };
            self.view_did_load();
        }

        #[method(viewWillAppear:)]
        fn _view_will_appear(&self, animated: bool) {
            self.view_will_appear();
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewWillAppear: animated] };
        }

        #[method(viewDidAppear:)]
        fn _view_did_appear(&self, animated: bool) {
            self.view_did_appear();
            // Docs say to call super
            let _: () = unsafe { msg_send![super(self), viewDidAppear: animated] };
        }
    }

    // Storyboard
    // See storyboard_connections.h
    unsafe impl AddController {
        // #[method(setTableView:)]
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

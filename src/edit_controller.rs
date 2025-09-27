use std::cell::OnceCell;
use std::error::Error;
use std::time::Duration;

use block2::{Block, RcBlock};
use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send, DefinedClass as _, Message};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSArray, NSBundle, NSCoder, NSIndexPath, NSInteger,
    NSObjectProtocol, NSString,
};
use objc2_ui_kit::{
    NSIndexPathUIKitAdditions, UIAction, UIButton, UILabel, UIMenu, UIMenuElementState,
    UIMenuOptions, UIScrollViewDelegate, UISegmentedControl, UITableView, UITableViewCell,
    UITableViewDataSource, UITableViewDelegate, UITextField, UIViewController,
};
use ruffle_core::{LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
use ruffle_frontend_utils::player_options::PlayerOptions;
use ruffle_render::quality::StageQuality;

use crate::storage::Movie;

#[derive(Clone, Copy, Debug)]
enum FormElement {
    Name,
    String {
        label: &'static str,
        text: fn(&PlayerOptions) -> Option<String>,
        write_if_set: fn(&mut PlayerOptions, &str) -> Result<(), Box<dyn Error>>,
    },
    Select {
        label: &'static str,
        variants: &'static [&'static str],
        enabled_variant: fn(&PlayerOptions) -> Option<&'static str>,
        write_if_set: fn(&mut PlayerOptions, &str) -> Result<(), Box<dyn Error>>,
    },
    Bool {
        label: &'static str,
        value: fn(&PlayerOptions) -> Option<bool>,
        write_if_set: fn(&mut PlayerOptions, bool),
    },
}

// TODO: Localization
// Roughly matches PlayerOptions
const FORM: &[&[FormElement]] = &[
    // Required
    &[FormElement::Name],
    // General options
    &[
        FormElement::String {
            label: "Maximum execution duration (s)",
            text: |options| {
                options
                    .max_execution_duration
                    .map(|duration| duration.as_secs().to_string())
            },
            write_if_set: |options, value| {
                let value = value
                    .parse()
                    .map_err(|err| format!("invalid duration: {err}"))?;
                options.max_execution_duration = Some(Duration::from_secs(value));
                Ok(())
            },
        },
        FormElement::Select {
            label: "Quality",
            variants: &[
                "Low",
                "Medium",
                "High",
                "Best",
                "High (8x8)",
                "High (8x8) Linear",
                "High (16x16)",
                "High (16x16) Linear",
            ],
            enabled_variant: |options| {
                options.quality.map(|quality| match quality {
                    StageQuality::Low => "Low",
                    StageQuality::Medium => "Medium",
                    StageQuality::High => "High",
                    StageQuality::Best => "Best",
                    StageQuality::High8x8 => "High (8x8)",
                    StageQuality::High8x8Linear => "High (8x8) Linear",
                    StageQuality::High16x16 => "High (16x16)",
                    StageQuality::High16x16Linear => "High (16x16) Linear",
                })
            },
            write_if_set: |options, value| {
                options.quality = Some(match value {
                    "Low" => StageQuality::Low,
                    "Medium" => StageQuality::Medium,
                    "High" => StageQuality::High,
                    "Best" => StageQuality::Best,
                    "High (8x8)" => StageQuality::High8x8,
                    "High (8x8) Linear" => StageQuality::High8x8Linear,
                    "High (16x16)" => StageQuality::High16x16,
                    "High (16x16) Linear" => StageQuality::High16x16Linear,
                    _ => return Err("invalid stage quality".into()),
                });
                Ok(())
            },
        },
        FormElement::String {
            label: "Player version",
            text: |options| options.player_version.map(|version| version.to_string()),
            write_if_set: |options, value| {
                let value = value
                    .parse()
                    .map_err(|err| format!("invalid player version: {err}"))?;
                options.player_version = Some(value);
                Ok(())
            },
        },
        FormElement::Select {
            label: "Player runtime",
            variants: &["Flash Player", "Adobe AIR"],
            enabled_variant: |options| {
                options.player_runtime.map(|runtime| match runtime {
                    PlayerRuntime::FlashPlayer => "Flash Player",
                    PlayerRuntime::AIR => "Adobe AIR",
                })
            },
            write_if_set: |options, value| {
                options.player_runtime = Some(match value {
                    "Flash Player" => PlayerRuntime::FlashPlayer,
                    "Adobe AIR" => PlayerRuntime::AIR,
                    _ => return Err("invalid player runtime".into()),
                });
                Ok(())
            },
        },
        FormElement::String {
            label: "Custom framerate (fps)",
            text: |options| options.frame_rate.map(|rate: f64| rate.to_string()),
            write_if_set: |options, value| {
                let value = value
                    .parse()
                    .map_err(|err| format!("invalid framerate: {err}"))?;
                options.frame_rate = Some(value);
                Ok(())
            },
        },
    ],
    // Stage Alignment
    &[
        FormElement::Select {
            label: "Alignment",
            variants: &[
                "Center",
                "Top",
                "Bottom",
                "Left",
                "Right",
                "Top-Left",
                "Top-Right",
                "Bottom-Left",
                "Bottom-Right",
            ],
            enabled_variant: |options| {
                const CENTER: StageAlign = StageAlign::empty();
                const TOP_LEFT: StageAlign = StageAlign::TOP.union(StageAlign::LEFT);
                const TOP_RIGHT: StageAlign = StageAlign::TOP.union(StageAlign::RIGHT);
                const BOTTOM_LEFT: StageAlign = StageAlign::BOTTOM.union(StageAlign::LEFT);
                const BOTTOM_RIGHT: StageAlign = StageAlign::BOTTOM.union(StageAlign::RIGHT);
                options.align.map(|align| match align {
                    CENTER => "Center",
                    StageAlign::TOP => "Top",
                    StageAlign::BOTTOM => "Bottom",
                    StageAlign::LEFT => "Left",
                    StageAlign::RIGHT => "Right",
                    TOP_LEFT => "Top-Left",
                    TOP_RIGHT => "Top-Right",
                    BOTTOM_LEFT => "Bottom-Left",
                    BOTTOM_RIGHT => "Bottom-Right",
                    // Fallback
                    _ => "Center",
                })
            },
            write_if_set: |options, value| {
                options.align = Some(match value {
                    "Center" => StageAlign::empty(),
                    "Top" => StageAlign::TOP,
                    "Bottom" => StageAlign::BOTTOM,
                    "Left" => StageAlign::LEFT,
                    "Right" => StageAlign::RIGHT,
                    "Top-Left" => StageAlign::TOP.union(StageAlign::LEFT),
                    "Top-Right" => StageAlign::TOP.union(StageAlign::RIGHT),
                    "Bottom-Left" => StageAlign::BOTTOM.union(StageAlign::LEFT),
                    "Bottom-Right" => StageAlign::BOTTOM.union(StageAlign::RIGHT),
                    _ => return Err("invalid stage".into()),
                });
                Ok(())
            },
        },
        FormElement::Bool {
            label: "Force",
            value: |options| options.force_align,
            write_if_set: |options, value| options.force_align = Some(value),
        },
    ],
    // Scale mode
    &[
        FormElement::Select {
            label: "Scale mode",
            variants: &[
                "Unscaled (100%)",
                "Zoom to Fit",
                "Stretch to Fit",
                "Crop to Fit",
            ],
            enabled_variant: |options| {
                options.scale.map(|scale| match scale {
                    StageScaleMode::NoScale => "Unscaled (100%)",
                    StageScaleMode::ShowAll => "Zoom to Fit",
                    StageScaleMode::ExactFit => "Stretch to Fit",
                    StageScaleMode::NoBorder => "Crop to Fit",
                })
            },
            write_if_set: |options, value| {
                options.scale = Some(match value {
                    "Unscaled (100%)" => StageScaleMode::NoScale,
                    "Zoom to Fit" => StageScaleMode::ShowAll,
                    "Stretch to Fit" => StageScaleMode::ExactFit,
                    "Crop to Fit" => StageScaleMode::NoBorder,
                    _ => return Err("invalid scale mode".into()),
                });
                Ok(())
            },
        },
        FormElement::Bool {
            label: "Force",
            value: |options| options.force_scale,
            write_if_set: |options, value| options.force_scale = Some(value),
        },
    ],
    // Network settings
    &[
        FormElement::String {
            label: "Custom base URL",
            text: |options| options.base.as_ref().map(|url| url.to_string()),
            write_if_set: |options, value| {
                let value = value.parse().map_err(|err| format!("invalid URL: {err}"))?;
                options.base = Some(value);
                Ok(())
            },
        },
        FormElement::String {
            label: "Spoof SWF URL",
            text: |options| options.spoof_url.as_ref().map(|url| url.to_string()),
            write_if_set: |options, value| {
                let value = value.parse().map_err(|err| format!("invalid URL: {err}"))?;
                options.spoof_url = Some(value);
                Ok(())
            },
        },
        FormElement::String {
            label: "Referer URL",
            text: |options| options.referer.as_ref().map(|url| url.to_string()),
            write_if_set: |options, value| {
                let value = value.parse().map_err(|err| format!("invalid URL: {err}"))?;
                options.referer = Some(value);
                Ok(())
            },
        },
        FormElement::String {
            label: "Cookie",
            text: |options| options.cookie.clone(),
            write_if_set: |options, value| {
                options.cookie = Some(value.to_string());
                Ok(())
            },
        },
        FormElement::Bool {
            label: "Upgrade HTTP to HTTPS",
            value: |options| options.upgrade_to_https,
            write_if_set: |options, value| options.upgrade_to_https = Some(value),
        },
        FormElement::Select {
            label: "Load behaviour",
            variants: &["Streaming", "Delayed", "Blocking"],
            enabled_variant: |options| {
                options.load_behavior.map(|behaviour| match behaviour {
                    LoadBehavior::Streaming => "Streaming",
                    LoadBehavior::Delayed => "Delayed",
                    LoadBehavior::Blocking => "Blocking",
                })
            },
            write_if_set: |options, value| {
                options.load_behavior = Some(match value {
                    "Streaming" => LoadBehavior::Streaming,
                    "Delayed" => LoadBehavior::Delayed,
                    "Blocking" => LoadBehavior::Blocking,
                    _ => return Err("invalid load behaviour".into()),
                });
                Ok(())
            },
        },
        FormElement::Bool {
            label: "Dummy external interface",
            value: |options| options.dummy_external_interface,
            write_if_set: |options, value| options.dummy_external_interface = Some(value),
        },
    ],
    // Movie parameters are placed at the end
];

#[derive(Default, Debug)]
pub struct Ivars {
    table_view: OnceCell<Retained<UITableView>>,
    movie: OnceCell<Retained<Movie>>,
}

define_class!(
    #[unsafe(super(UIViewController))]
    #[name = "EditController"]
    #[ivars = Ivars]
    #[derive(Debug)]
    pub struct EditController;

    unsafe impl NSObjectProtocol for EditController {}

    /// UIViewController.
    impl EditController {
        #[unsafe(method_id(initWithNibName:bundle:))]
        fn _init_with_nib_name_bundle(
            this: Allocated<Self>,
            nib_name_or_nil: Option<&NSString>,
            nib_bundle_or_nil: Option<&NSBundle>,
        ) -> Retained<Self> {
            tracing::info!("edit init");
            let this = this.set_ivars(Ivars::default());
            unsafe {
                msg_send![super(this), initWithNibName: nib_name_or_nil, bundle: nib_bundle_or_nil]
            }
        }

        #[unsafe(method_id(initWithCoder:))]
        fn _init_with_coder(this: Allocated<Self>, coder: &NSCoder) -> Option<Retained<Self>> {
            tracing::info!("edit init");
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
    impl EditController {
        #[unsafe(method(setTableView:))]
        fn _set_table_view(&self, table_view: &UITableView) {
            tracing::trace!("edit set table view");
            self.ivars()
                .table_view
                .set(table_view.retain())
                .expect("only set table view once");
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UITableViewDataSource for EditController {
        #[unsafe(method(tableView:numberOfRowsInSection:))]
        fn tableView_numberOfRowsInSection(
            &self,
            _table_view: &UITableView,
            section: NSInteger,
        ) -> NSInteger {
            if FORM.len() == section as usize {
                let movie = self.ivars().movie.get().unwrap();
                movie.user_options().parameters.len() as NSInteger + 1
            } else {
                FORM[section as usize].len() as NSInteger
            }
        }

        #[unsafe(method(numberOfSectionsInTableView:))]
        fn numberOfSectionsInTableView(&self, _table_view: &UITableView) -> NSInteger {
            FORM.len() as NSInteger + 1
        }

        #[unsafe(method_id(tableView:cellForRowAtIndexPath:))]
        fn tableView_cellForRowAtIndexPath(
            &self,
            table_view: &UITableView,
            index_path: &NSIndexPath,
        ) -> Retained<UITableViewCell> {
            self.cell_at_index_path(table_view, index_path)
        }
    }

    unsafe impl UIScrollViewDelegate for EditController {}

    unsafe impl UITableViewDelegate for EditController {}
);

impl EditController {
    pub fn setup_movie(&self, movie: &Movie) {
        self.ivars().movie.set(movie.retain()).unwrap();
    }

    fn view_did_load(&self) {
        tracing::info!("edit viewDidLoad");
    }

    fn view_will_appear(&self) {
        tracing::info!("edit viewWillAppear:");
    }

    fn view_did_appear(&self) {
        tracing::info!("edit viewDidAppear:");

        // Do the same thing as UITableViewController, flash the scroll bar
        let table = self.ivars().table_view.get().expect("table view");
        table.flashScrollIndicators();
    }

    fn cell_at_index_path(
        &self,
        table_view: &UITableView,
        index_path: &NSIndexPath,
    ) -> Retained<UITableViewCell> {
        let mtm = MainThreadMarker::from(self);
        let movie = self.ivars().movie.get().unwrap();
        let options = movie.user_options();
        let section = index_path.section() as usize;
        let row = index_path.row() as usize;
        if FORM.len() == section {
            if options.parameters.len() == row {
                return table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                    ns_string!("movie-parameter-add"),
                    index_path,
                );
            }

            let (param, value) = &options.parameters[row];
            let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                ns_string!("movie-parameter"),
                index_path,
            );
            let subviews = cell.contentView().subviews();
            let ui_param = subviews.objectAtIndex(1).downcast::<UITextField>().unwrap();
            ui_param.setText(Some(&NSString::from_str(param)));
            let ui_value = subviews.objectAtIndex(2).downcast::<UITextField>().unwrap();
            ui_value.setText(Some(&NSString::from_str(value)));

            return cell;
        }

        match FORM[section][row] {
            FormElement::Name => {
                let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                    ns_string!("root-name"),
                    index_path,
                );
                let input = cell
                    .contentView()
                    .subviews()
                    .objectAtIndex(0)
                    .downcast::<UITextField>()
                    .unwrap();
                input.setText(Some(&movie.cachedName()));
                cell
            }
            FormElement::String { label, text, .. } => {
                let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                    ns_string!("string"),
                    index_path,
                );
                let subviews = cell.contentView().subviews();

                let ui_label = subviews.objectAtIndex(0).downcast::<UILabel>().unwrap();
                ui_label.setText(Some(&NSString::from_str(label)));

                let input = subviews.objectAtIndex(1).downcast::<UITextField>().unwrap();
                input.setText(text(&options).map(|s| NSString::from_str(&s)).as_deref());
                cell
            }
            FormElement::Select {
                label,
                variants,
                enabled_variant,
                ..
            } => {
                let cell = table_view.dequeueReusableCellWithIdentifier_forIndexPath(
                    ns_string!("select"),
                    index_path,
                );
                let subviews = cell.contentView().subviews();

                let ui_label = subviews.objectAtIndex(0).downcast::<UILabel>().unwrap();
                ui_label.setText(Some(&NSString::from_str(label)));

                // Set menu
                let enabled_variant = enabled_variant(&options);
                let button = subviews.objectAtIndex(1).downcast::<UIButton>().unwrap();
                // We have to use UIAction here, UICommand seems to be broken
                let block = RcBlock::new(|_| {});
                let block_ptr: *const Block<_> = &*block;
                let default_item =
                    unsafe { UIAction::actionWithHandler(block_ptr.cast_mut(), mtm) };
                default_item.setTitle(ns_string!("Default"));
                if enabled_variant.is_none() {
                    default_item.setState(UIMenuElementState::On);
                }

                let children: Retained<NSArray<_>> = variants
                    .iter()
                    .map(|title| {
                        let cmd = unsafe { UIAction::actionWithHandler(block_ptr.cast_mut(), mtm) };
                        cmd.setTitle(&NSString::from_str(title));
                        if enabled_variant == Some(title) {
                            cmd.setState(UIMenuElementState::On);
                        }
                        Retained::into_super(cmd)
                    })
                    .collect();
                button.setMenu(Some(
                    &UIMenu::menuWithTitle_image_identifier_options_children(
                        ns_string!(""),
                        None,
                        None,
                        UIMenuOptions::SingleSelection,
                        &NSArray::from_slice(&[
                            &**default_item,
                            &*UIMenu::menuWithTitle_image_identifier_options_children(
                                ns_string!(""),
                                None,
                                None,
                                UIMenuOptions::DisplayInline | UIMenuOptions::SingleSelection,
                                &children,
                                mtm,
                            ),
                        ]),
                        mtm,
                    ),
                ));
                cell
            }
            FormElement::Bool { label, value, .. } => {
                let cell = table_view
                    .dequeueReusableCellWithIdentifier_forIndexPath(ns_string!("bool"), index_path);
                let subviews = cell.contentView().subviews();

                let ui_label = subviews.objectAtIndex(0).downcast::<UILabel>().unwrap();
                ui_label.setText(Some(&NSString::from_str(label)));

                let control = subviews
                    .objectAtIndex(1)
                    .downcast::<UISegmentedControl>()
                    .unwrap();
                control.setSelectedSegmentIndex(match value(&options) {
                    None => 0,
                    Some(false) => 1,
                    Some(true) => 2,
                });
                cell
            }
        }
    }

    // fn get_data(&self) -> (Retained<NSString>, PlayerOptions) {
    //     let table_view = self.ivars().table_view.get().unwrap();
    //     let path = NSIndexPath::new();
    //     for (i, section) in FORM.iter().enumerate() {
    //         let cell = table_view.cellForRowAtIndexPath();
    //     }
    //
    //     // Movie parameters
    // }
}

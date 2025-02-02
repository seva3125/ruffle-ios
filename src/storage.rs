//! Interface for storing user data with CoreData.
//!
//! Overview:
//! - Movie
//!   - link
//!   - userOptions
//!   - movieData
//!     - key/value
//!
//! External methods will be filled in dynamically by CoreData.
//!
//! TODO:
//! - Figure out sync failures.
//! - Better error handling.
//! - Use `define_class!` once we can create ivars with specific names in
//!   that (required for NSManagedObject to work).
//!
//! To generate data model interface to compare with, modify .xcdatamodeld and
//! set codegen = Class definition on every entity. Then run:
//! /Applications/Xcode.app/Contents/Developer/usr/bin/momc --action generate ./Ruffle.xcdatamodeld storage
#![allow(non_snake_case)]

use std::ops::Deref;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::OnceLock;

use block2::RcBlock;
use objc2::encode::{Encoding, RefEncode};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyClass, ClassBuilder};
use objc2::{extern_methods, msg_send, AllocAnyThread, ClassType, Message};
use objc2_core_data::{
    NSFetchRequest, NSFetchedResultsController, NSManagedObject, NSManagedObjectContext,
    NSPersistentContainer, NSPersistentStoreDescription,
};
use objc2_foundation::{
    ns_string, NSArray, NSData, NSError, NSSet, NSSortDescriptor, NSString, NSURL,
};
use ruffle_core::backend::storage::StorageBackend;
use ruffle_frontend_utils::bundle::Bundle;
use ruffle_frontend_utils::player_options::PlayerOptions;

/// The data relevant for an SWF movie / a Ruffle Bundle.
#[repr(transparent)]
#[derive(Debug)]
pub struct Movie {
    superclass: NSManagedObject,
}

unsafe impl RefEncode for Movie {
    const ENCODING_REF: Encoding = NSManagedObject::ENCODING_REF;
}

unsafe impl Message for Movie {}

impl Deref for Movie {
    type Target = NSManagedObject;

    fn deref(&self) -> &Self::Target {
        &self.superclass
    }
}

impl Movie {
    pub fn class() -> &'static AnyClass {
        static CLS: OnceLock<&'static AnyClass> = OnceLock::new();

        CLS.get_or_init(|| {
            eprintln!("create Movie class");
            let mut builder = ClassBuilder::new(c"Movie", NSManagedObject::class()).unwrap();

            // FIXME: Deallocation of these in `dealloc`.
            builder.add_ivar::<*mut NSURL>(c"link");
            builder.add_ivar::<*mut NSString>(c"cachedName");
            builder.add_ivar::<*mut NSData>(c"userOptions");
            builder.add_ivar::<*mut NSSet<MovieData>>(c"movieData");

            builder.register()
        })
    }

    // NSManagedObject initializers.

    fn initWithContext(this: Allocated<Self>, moc: &NSManagedObjectContext) -> Retained<Self> {
        unsafe { msg_send![this, initWithContext: moc] }
    }

    fn fetchRequest() -> Retained<NSFetchRequest<Self>> {
        unsafe { msg_send![Self::class(), fetchRequest] }
    }

    // Properties
    extern_methods!(
        /// Reference/bookmark to a Ruffle Bundle or SWF.
        /// - Either a bookmarked link to the actual bundle/SWF stored on user's device.
        /// - Or http/https link to externally stored bundle/SWF.
        #[unsafe(method(link))]
        pub fn link(&self) -> Retained<NSURL>;

        #[unsafe(method(setLink:))]
        pub fn setLink(&self, value: &NSURL);

        /// A cached value of the name of the bundle/SWF. Allows us to avoid
        /// reading the link when displaying the list of movies.
        #[unsafe(method(cachedName))]
        pub fn cachedName(&self) -> Retained<NSString>;

        #[unsafe(method(setCachedName:))]
        pub fn setCachedName(&self, value: &NSString);

        /// Any user-specified settings (overrides the Ruffle Bundle's preconfigured settings).
        #[unsafe(method(userOptions))]
        fn _userOptions(&self) -> Retained<NSData>;

        #[unsafe(method(setUserOptions:))]
        fn _setUserOptions(&self, value: &NSData);

        /// Data the SWF itself may have stored (the `.sol` key-value store).
        #[unsafe(method(movieData))]
        pub fn movieData(&self) -> Retained<NSSet<MovieData>>;

        #[unsafe(method(setMovieData:))]
        pub fn setMovieData(&self, values: &NSSet<MovieData>);
    );

    pub fn user_options(&self) -> PlayerOptions {
        // TODO: Convert from binary data in _userOptions.
        // Maybe using serde?
        PlayerOptions::default()
    }

    pub fn set_user_options(&self, _options: &PlayerOptions) {
        // TODO: Convert to binary data.
        self._setUserOptions(&NSData::with_bytes(b"{}"));
    }

    // Perhaps: `cachedName`, to allow easily finding relevant settings for an SWF in case the user deleted?

    // Generated accessors
    extern_methods!(
        #[unsafe(method(addMovieDataObject:))]
        pub fn addMovieDataObject(&self, value: &MovieData);

        #[unsafe(method(removeMovieDataObject:))]
        pub fn removeMovieDataObject(&self, value: &MovieData);

        #[unsafe(method(addMovieData:))]
        pub fn addMovieData(&self, values: &NSSet<MovieData>);

        #[unsafe(method(removeMovieData:))]
        pub fn removeMovieData(&self, values: &NSSet<MovieData>);
    );
}

/// Key/value pairs of data that the movie itself wants to store (.sol).
///
/// Intended invariant: Keys are unique.
#[repr(transparent)]
#[derive(Debug)]
pub struct MovieData {
    superclass: NSManagedObject,
}

unsafe impl RefEncode for MovieData {
    const ENCODING_REF: Encoding = NSManagedObject::ENCODING_REF;
}

unsafe impl Message for MovieData {}

impl Deref for MovieData {
    type Target = NSManagedObject;

    fn deref(&self) -> &Self::Target {
        &self.superclass
    }
}

impl MovieData {
    pub fn class() -> &'static AnyClass {
        static CLS: OnceLock<&'static AnyClass> = OnceLock::new();

        CLS.get_or_init(|| {
            eprintln!("create MovieData class");
            let mut builder = ClassBuilder::new(c"MovieData", NSManagedObject::class()).unwrap();

            // FIXME: Deallocation of these in `dealloc`.
            builder.add_ivar::<*mut NSString>(c"key");
            builder.add_ivar::<*mut NSData>(c"value");

            builder.register()
        })
    }

    // NSManagedObject initializers.

    fn initWithContext(this: Allocated<Self>, moc: &NSManagedObjectContext) -> Retained<Self> {
        unsafe { msg_send![this, initWithContext: moc] }
    }

    fn fetchRequest() -> Retained<NSFetchRequest<Self>> {
        unsafe { msg_send![Self::class(), fetchRequest] }
    }

    // Properties
    extern_methods!(
        #[unsafe(method(key))]
        pub fn key(&self) -> Retained<NSString>;

        #[unsafe(method(setKey:))]
        pub fn setKey(&self, value: &NSString);

        #[unsafe(method(value))]
        pub fn value(&self) -> Retained<NSData>;

        #[unsafe(method(setValue:))]
        pub fn setValue(&self, value: &NSData);

        #[unsafe(method(movie))]
        pub fn movie(&self) -> Retained<Movie>;

        #[unsafe(method(setMovie:))]
        pub fn setMovie(&self, value: &Movie);
    );
}

#[derive(Debug, Clone)]
pub struct MovieStorageBackend {
    movie: Retained<Movie>,
}

impl MovieStorageBackend {
    fn lookup_data(&self, key: &NSString) -> Option<Retained<MovieData>> {
        // TODO: Do this lookup on the CoreData model directly?
        // Maybe using NSPredicate?
        self.movie
            .movieData()
            .iter()
            .find(|data| &*data.key() == key)
    }
}

impl StorageBackend for MovieStorageBackend {
    fn get(&self, name: &str) -> Option<Vec<u8>> {
        let key = NSString::from_str(name);
        let data = self.lookup_data(&key)?;
        Some(data.value().to_vec())
    }

    fn put(&mut self, name: &str, value: &[u8]) -> bool {
        let key = NSString::from_str(name);
        let value = NSData::with_bytes(value);
        if let Some(existing) = self.lookup_data(&key) {
            existing.setValue(&value);
        } else {
            let data = unsafe { msg_send![MovieData::class(), alloc] };
            let data = MovieData::initWithContext(data, unsafe { &container().viewContext() });
            data.setKey(&key);
            data.setValue(&value);
            self.movie.addMovieDataObject(&data);
        }

        // Flush changes to disk.
        match unsafe { container().viewContext().save() } {
            Ok(()) => true,
            Err(err) => {
                eprintln!("failed saving key {name:?}: {err}");
                false
            }
        }
    }

    fn remove_key(&mut self, name: &str) {
        let key = NSString::from_str(name);
        if let Some(existing) = self.lookup_data(&key) {
            unsafe { container().viewContext().deleteObject(&existing) };
        }

        // Flush changes to disk.
        unsafe { container().viewContext().save() }.unwrap_or_else(|err| {
            eprintln!("failed removing key {name:?}: {err}");
        })
    }
}

static PERSISTENT: OnceLock<Retained<NSPersistentContainer>> = OnceLock::new();

pub fn setup() {
    let persistent = PERSISTENT.get_or_init(|| unsafe {
        NSPersistentContainer::persistentContainerWithName(ns_string!("Ruffle"))
    });

    let block = RcBlock::new(
        |descriptor: NonNull<NSPersistentStoreDescription>, err: *mut NSError| {
            if let Some(err) = unsafe { err.as_ref() } {
                panic!("failed loading: {err}");
            }
            let descriptor = unsafe { descriptor.as_ref() };
            tracing::info!("loading {descriptor:?}");
        },
    );
    unsafe { persistent.loadPersistentStoresWithCompletionHandler(&block) };

    tracing::info!("finished storage setup");
}

fn container() -> &'static NSPersistentContainer {
    PERSISTENT
        .get()
        .expect("NSPersistentContainer must be initialized")
}

fn access_security_scoped_resource(url: &NSURL) -> Option<impl Drop + '_> {
    struct OnDrop<'a>(&'a NSURL);

    impl Drop for OnDrop<'_> {
        fn drop(&mut self) {
            unsafe { self.0.stopAccessingSecurityScopedResource() };
        }
    }

    if unsafe { url.startAccessingSecurityScopedResource() } {
        Some(OnDrop(url))
    } else {
        None
    }
}

fn url_to_path(url: &NSURL) -> PathBuf {
    // TODO: Use fileSystemRepresentation?
    let path = unsafe { url.filePathURL().unwrap().path().unwrap() };
    PathBuf::from(path.to_string())
}

pub fn add_local_movie(url: &NSURL) {
    assert!(unsafe { url.isFileURL() }, "was not file URL");

    let Some(access) = access_security_scoped_resource(url) else {
        panic!("failed accessing NSURL: {url:?}");
    };
    // TODO: Load SWFs here too.
    let Ok(bundle) = Bundle::from_path(url_to_path(url)).map_err(|err| {
        eprintln!("failed loading bundle {url:?}: {err}");
    }) else {
        return;
    };
    drop(access);

    for warning in bundle.warnings() {
        eprintln!("bundle warning: {warning}");
    }

    let movie = unsafe { msg_send![Movie::class(), alloc] };
    let movie = Movie::initWithContext(movie, unsafe { &container().viewContext() });
    movie.setLink(&url);
    movie.setCachedName(&NSString::from_str(&bundle.information().name));
    movie.set_user_options(&PlayerOptions::default());

    // Flush changes to disk.
    unsafe { container().viewContext().save() }.unwrap_or_else(|err| {
        eprintln!("failed adding movie {url:?}: {err}");
    })
}

pub fn all_movies() -> Retained<NSFetchedResultsController<Movie>> {
    unsafe {
        let fetch_request = Movie::fetchRequest();

        let cached_name_descriptor =
            NSSortDescriptor::sortDescriptorWithKey_ascending(Some(ns_string!("cachedName")), true);
        let link_descriptor = NSSortDescriptor::sortDescriptorWithKey_ascending(
            Some(ns_string!("link.lastPathComponent")),
            true,
        );
        let sort_descriptors =
            NSArray::from_retained_slice(&[cached_name_descriptor, link_descriptor]);
        fetch_request.setSortDescriptors(Some(&sort_descriptors));

        NSFetchedResultsController::initWithFetchRequest_managedObjectContext_sectionNameKeyPath_cacheName(
            NSFetchedResultsController::alloc(),
            &fetch_request,
            &container().viewContext(),
            None, // No sectioning
            None, // No cache
        )
    }
}

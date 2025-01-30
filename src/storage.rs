//! Interface for storing user data with CoreData.
//!
//! Overview:
//! - Movie
//!   - link
//!   - userSettings
//!   - movieData
//!     - key/value
//!
//! External methods will be filled in dynamically by CoreData.
//!
//! TODO:
//! - Figure out sync failures.
//!
//! To generate data model interface to compare with, modify .xcdatamodeld and
//! set codegen = Class definition on every entity. Then run:
//! /Applications/Xcode.app/Contents/Developer/usr/bin/momc --action generate ./Ruffle.xcdatamodeld storage
#![allow(non_snake_case)]

use objc2::rc::{Allocated, Retained};
use objc2::{define_class, extern_methods, AllocAnyThread};
use objc2_core_data::{NSFetchRequest, NSManagedObject, NSManagedObjectContext};
use objc2_foundation::{ns_string, NSData, NSSet, NSString, NSURL};
use ruffle_core::backend::storage::StorageBackend;

define_class!(
    /// The data relevant for an SWF movie / a Ruffle Bundle.
    #[unsafe(super(NSManagedObject))]
    #[name = "Movie"]
    #[derive(Debug)]
    pub struct Movie;

    /// NSManagedObject override.
    impl Movie {
        #[unsafe(method_id(fetchRequest))]
        fn fetchRequest() -> Retained<NSFetchRequest<Self>> {
            unsafe { NSFetchRequest::fetchRequestWithEntityName(ns_string!("Movie")) }
        }
    }
);

impl Movie {
    // NSManagedObject initializers.
    extern_methods!(
        #[unsafe(method(initWithContext:))]
        pub fn initWithContext(
            this: Allocated<Self>,
            moc: &NSManagedObjectContext,
        ) -> Retained<Self>;
    );

    // Properties
    extern_methods!(
        /// Reference/bookmark to a Ruffle Bundle or SWF.
        /// - Either a bookmarked link to the actual bundle/SWF stored on user's device.
        /// - Or http/https link to externally stored bundle/SWF.
        #[unsafe(method(link))]
        pub fn link(&self) -> Retained<NSURL>;

        #[unsafe(method(setLink:))]
        pub fn setLink(&self, value: &NSURL);

        /// Any user-specified settings (overrides the Ruffle Bundle's preconfigured settings).
        #[unsafe(method(userSettings))]
        pub fn userSettings(&self) -> Retained<NSData>;

        #[unsafe(method(setUserSettings:))]
        pub fn setUserSettings(&self, value: &NSData);

        /// Data the SWF itself may have stored (the `.sol` key-value store).
        #[unsafe(method(movieData))]
        pub fn movieData(&self) -> Retained<NSSet<MovieData>>;

        #[unsafe(method(setMovieData:))]
        pub fn setMovieData(&self, values: &NSSet<MovieData>);
    );

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

define_class!(
    /// Key/value pairs of data that the movie itself wants to store (.sol).
    ///
    /// Intended invariant: Keys are unique.
    #[unsafe(super(NSManagedObject))]
    #[name = "MovieData"]
    #[derive(Debug)]
    pub struct MovieData;

    /// NSManagedObject override.
    impl MovieData {
        #[unsafe(method_id(fetchRequest))]
        fn fetchRequest() -> Retained<NSFetchRequest<Self>> {
            unsafe { NSFetchRequest::fetchRequestWithEntityName(ns_string!("MovieData")) }
        }
    }
);

impl MovieData {
    // NSManagedObject initializers.
    extern_methods!(
        #[unsafe(method(initWithContext:))]
        pub fn initWithContext(
            this: Allocated<Self>,
            moc: &NSManagedObjectContext,
        ) -> Retained<Self>;
    );

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
    context: Retained<NSManagedObjectContext>,
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
            let data = MovieData::initWithContext(MovieData::alloc(), &self.context);
            data.setKey(&key);
            data.setValue(&value);
            self.movie.addMovieDataObject(&data);
        }
        true
    }

    fn remove_key(&mut self, name: &str) {
        let key = NSString::from_str(name);
        if let Some(existing) = self.lookup_data(&key) {
            self.movie.removeMovieDataObject(&existing);
        }
    }
}

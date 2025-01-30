# The Ruffle Flash Player emulator on iOS

Work in progress.

See [ruffle.rs](https://ruffle.rs/) for a general introduction.


## Design choices

A normal person might have wrapped the Rust in some `extern "C" fn`s, and then used SwiftUI, or at least Objective-C for the UI shell. I would probably recommend that for most use-cases.

I'm developing [`objc2`](https://github.com/madsmtm/objc2) though, and I want to improve the user-interface of that, so I decided to be a bit unortodox, and do everything in Rust.

## Testing

Run the core player on Mac Catalyst with:
```
cargo bundle --target=aarch64-apple-ios-macabi --bin run_swf && ./target/aarch64-apple-ios-macabi/debug/bundle/ios/Ruffle.app/run_swf
```

## UI

Similar to https://getutm.app/, we should have:
- A library of "installed" SWFs/bundles/saved links, editable.
- When selecting an SWF, the navigation bar at the top shows various options
  - Opening keyboard (maybe?)
  - Context menu "play, rewind, forward, back, etc."?
  - Allow changing between scale
  - Back button to go back to library
- "Add" and "edit" are two different flows, and should show two different UIs
  - "Add" doesn't have to show all the extra settings; it is only about getting the file. The user can edit it later.

## Library item settings

Settings are stored per Ruffle Bundle.

- `PlayerOptions`
  - https://github.com/ruffle-rs/ruffle/blob/master/frontend-utils/src/bundle/README.md#player
- Inputs:
  - Configurable
  - Swipe for arrow keys?
  - https://openemu.org/ does it pretty well, equivalent for iOS?
- Custom name?
- Custom image?


## Storage

We do not store Ruffle Bundles / SWFs, the user is responsible for doing that themselves in the Files app. We only store "bookmarks" to these, to allow easily re-opening from within the app, and to store user data.

This can be synced to iCloud, though the user may have to re-select the referenced Ruffle Bundle (in case it was stored locally, and not in iCloud).

Goal: Be backwards and forwards compatible with new versions of the Ruffle app.
- Upheld for [Ruffle Bundles](https://discord.com/channels/610531541889581066/1225519553916829736/1232031955751665777).
- Should also be fine for user settings.

See [src/storage.rs] for implementation.


## Terminology

What do we call an SWF / a Ruffle Bundle? "Game"? "Movie"? "SWF"? "Flash Animation"?

Internally: "movie".


## Plan

1. Get the Ruffle UI running in a `UIView`
2. Wire up some way to start it using an SWF on the local device


## TODO

- Set `idleTimerDisabled` at the appropriate time
- Use white for labels, orange for buttons
- Add settings button in library item
- Add quicklook thumbnail generator app extension
- Figure out what `UIDocument` actually does?

## Choices

- Intentionally use `public.app-category.games` to get better performance ("Game Mode" on macOS).
  - This is not necessarily the correct choice for Ruffle, but it's the closest.
- It doesn't make sense to have root settings like in the desktop version
- No tab bar, not really desired, since we generally want the SWF's UI to fill most of the screen
  - Though if we decide to add an easy way to download from "trusted" sources, we could add a tab bar for that
- A navigation bar is useful though
  - To display some settings for the current swf
  - To go back to library
  - Hide when entering full screen?

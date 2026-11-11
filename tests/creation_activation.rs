// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Creating a webview does not activate the application: the window that holds the webview decides its activation when it
//! is shown, so a webview that a background application creates leaves the active application in front.
//!
//! The test counts the calls of `-[NSApplication activate]` and `-[NSApplication activateIgnoringOtherApps:]` while it creates a
//! webview that fills a window and a child webview. It runs on the main thread (`harness = false`), as AppKit requires.

#[cfg(target_os = "macos")]
fn main() {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use objc2::{
    ffi::{class_getInstanceMethod, method_setImplementation},
    runtime::{AnyClass, AnyObject, Imp, Sel},
    sel,
  };
  use tao::{event_loop::EventLoop, window::WindowBuilder};
  use wry::{
    dpi::{LogicalPosition, LogicalSize},
    Rect, WebViewBuilder,
  };

  static ACTIVATIONS: AtomicUsize = AtomicUsize::new(0);

  extern "C-unwind" fn activate(_: &AnyObject, _: Sel) {
    ACTIVATIONS.fetch_add(1, Ordering::SeqCst);
  }

  extern "C-unwind" fn activate_ignoring_other_apps(_: &AnyObject, _: Sel, _: bool) {
    ACTIVATIONS.fetch_add(1, Ordering::SeqCst);
  }

  // Replaces a method of NSApplication with a counter, so a call activates nothing and is counted.
  fn count(selector: Sel, imp: Imp) {
    let class = AnyClass::get(c"NSApplication").expect("NSApplication is a class");
    let method = unsafe { class_getInstanceMethod(class, selector) };
    assert!(
      !method.is_null(),
      "NSApplication has no method {selector:?}"
    );
    unsafe { method_setImplementation(method as *mut _, imp) };
  }

  count(sel!(activate), unsafe {
    std::mem::transmute::<extern "C-unwind" fn(&AnyObject, Sel), Imp>(activate)
  });
  count(sel!(activateIgnoringOtherApps:), unsafe {
    std::mem::transmute::<extern "C-unwind" fn(&AnyObject, Sel, bool), Imp>(
      activate_ignoring_other_apps,
    )
  });

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new()
    .with_visible(false)
    .with_focused(false)
    .build(&event_loop)
    .expect("the window is created");

  let _filled = WebViewBuilder::new()
    .with_html("<p>filled</p>")
    .build(&window)
    .expect("the webview that fills the window is created");
  assert_eq!(
    ACTIVATIONS.load(Ordering::SeqCst),
    0,
    "creating the webview that fills a window activated the application"
  );

  let _child = WebViewBuilder::new()
    .with_html("<p>child</p>")
    .with_bounds(Rect {
      position: LogicalPosition::new(0, 0).into(),
      size: LogicalSize::new(100, 100).into(),
    })
    .build_as_child(&window)
    .expect("the child webview is created");
  assert_eq!(
    ACTIVATIONS.load(Ordering::SeqCst),
    0,
    "creating a child webview activated the application"
  );
  println!("creating webviews activated the application 0 times");
}

#[cfg(not(target_os = "macos"))]
fn main() {}

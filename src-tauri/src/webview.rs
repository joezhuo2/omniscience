//! WebView2 memory tuning.

use tauri::WebviewWindow;

/// Asks WebView2 to trim memory (`Low`) while the overlay is collapsed or hidden, and to return
/// to `Normal` while the dashboard is open. A no-op on other platforms or older runtimes.
#[cfg(windows)]
pub fn set_memory_low(window: &WebviewWindow, low: bool) {
    let result = window.with_webview(move |webview| {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL, ICoreWebView2_19,
        };
        use windows_core::Interface;
        let level = if low {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
        } else {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
        };
        // SAFETY: called on the webview's thread by `with_webview`; COM interfaces are
        // reference-counted wrappers.
        unsafe {
            let Ok(core) = webview.controller().CoreWebView2() else {
                return;
            };
            if let Ok(core19) = core.cast::<ICoreWebView2_19>() {
                let _ = core19.SetMemoryUsageTargetLevel(level);
            }
        }
    });
    if let Err(e) = result {
        eprintln!("webview memory level: {e}");
    }
}

#[cfg(not(windows))]
pub fn set_memory_low(_window: &WebviewWindow, _low: bool) {}

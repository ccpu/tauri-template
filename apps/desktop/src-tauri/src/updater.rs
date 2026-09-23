//! Asking GitHub whether there is a newer build of this app.
//!
//! The updater plugin fetches one manifest, compares the version in it with
//! this build's, and verifies every artifact it downloads against the public
//! key in `tauri.conf.json`. Nothing but the version is sent.
//!
//! A development build never asks: it cannot replace itself, so the check
//! would only ever end in a dialog that could do nothing.
//!
//! See `docs/updater.md` for generating the key pair and setting the secrets
//! the release workflow signs with.

use serde_json::Value;
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::UpdaterExt;

/// Whether the updater has been given a key to check signatures against.
///
/// Without one the plugin cannot verify anything it downloads, so it is not
/// registered at all and this module stands down — which is the state the
/// template ships in, and the state a fork is in until it sets up signing.
pub fn is_configured(config: &tauri::Config) -> bool {
    config
        .plugins
        .0
        .get("updater")
        .and_then(|updater| updater.get("pubkey"))
        .and_then(Value::as_str)
        .is_some_and(|pubkey| !pubkey.trim().is_empty())
}

/// Checks once, in the background, and offers whatever it finds.
///
/// Replace the dialogs with your own UI when you have one: the only parts that
/// matter are `updater.check()` and `update.download_and_install()`.
pub fn check_on_startup(app: &AppHandle) {
    let app = app.clone();

    tauri::async_runtime::spawn(async move {
        if tauri::is_dev() {
            return;
        }

        match app.updater() {
            Ok(updater) => match updater.check().await {
                Ok(Some(update)) => offer(&app, update).await,
                Ok(None) => {}
                Err(error) => eprintln!("Failed to check for updates: {error}"),
            },
            Err(error) => eprintln!("The updater is not available: {error}"),
        }
    });
}

async fn offer(app: &AppHandle, update: tauri_plugin_updater::Update) {
    let version = update.version.clone();
    let (sender, mut receiver) = tauri::async_runtime::channel(1);

    app.dialog()
        .message(format!(
            "Version {version} is available. Download and install it now? The app will \
             restart when it is done."
        ))
        .title("An update is available")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install".into(),
            "Not now".into(),
        ))
        .show(move |answer| {
            let _ = sender.blocking_send(answer);
        });

    if receiver.recv().await != Some(true) {
        return;
    }

    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => app.restart(),
        Err(error) => eprintln!("Failed to install the update: {error}"),
    }
}

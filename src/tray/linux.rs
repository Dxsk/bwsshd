//! Linux: a StatusNotifierItem over D-Bus (KDE natively, GNOME with the
//! AppIndicator extension). The watch loop runs on the calling thread.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

use ksni::blocking::TrayMethods;
use ksni::menu::StandardItem;
use ksni::{Icon, MenuItem};

use super::{Notify, WAITING, icon_rgba, label, open_config, version_line};

struct Tray {
    status: String,
    resync: Sender<()>,
    out: PathBuf,
}

impl ksni::Tray for Tray {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        "bwsshd".into()
    }

    fn title(&self) -> String {
        "bwsshd".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let mut data = icon_rgba(32);
        // SNI wants ARGB in network byte order.
        for px in data.chunks_exact_mut(4) {
            px.rotate_right(1);
        }
        vec![Icon {
            width: 32,
            height: 32,
            data,
        }]
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: format!("bwsshd: {}", self.status),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: format!("bwsshd {}", version_line()),
                enabled: false,
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: self.status.clone(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Resync now".into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.resync.send(());
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Open generated config".into(),
                activate: Box::new(|t: &mut Self| open_config(&t.out)),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit bwsshd".into(),
                activate: Box::new(|_: &mut Self| {
                    info!("quit from the tray");
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn run<F>(out: PathBuf, watch: F) -> Result<(), String>
where
    F: FnOnce(Receiver<()>, Notify) + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let tray = Tray {
        status: WAITING.into(),
        resync: tx,
        out,
    };
    // At login the panel may come up after us: wait for it instead of failing.
    let handle = tray
        .assume_sni_available(true)
        .spawn()
        .map_err(|e| format!("tray: {e}"))?;
    watch(
        rx,
        Box::new(move |s| {
            let text = label(&s, Instant::now());
            handle.update(|t| t.status = text);
        }),
    );
    Ok(())
}

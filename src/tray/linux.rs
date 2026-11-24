//! Linux: a StatusNotifierItem over D-Bus (KDE natively, GNOME with the
//! AppIndicator extension). The watch loop runs on the calling thread.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use ksni::blocking::TrayMethods;
use ksni::menu::{RadioGroup, RadioItem, StandardItem, SubMenu};
use ksni::{Icon, MenuItem};

use super::{
    Control, Notify, PRESETS, Status, WAITING, every, icon_rgba, label, open_config, version_line,
};

struct Tray {
    status: String,
    paused: bool,
    interval: u64,
    control: Sender<Control>,
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
        let mut data = icon_rgba(32, self.paused);
        // SNI wants ARGB in network byte order.
        for px in data.as_chunks_mut::<4>().0 {
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
                    let _ = t.control.send(Control::Resync);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: if self.paused {
                    "Resume auto sync"
                } else {
                    "Pause auto sync"
                }
                .into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.control.send(Control::Pause(!t.paused));
                }),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: format!("Sync every {}", every(self.interval)),
                submenu: vec![
                    RadioGroup {
                        // Past the end when --interval is not a preset: none checked.
                        selected: PRESETS
                            .iter()
                            .position(|&s| s == self.interval)
                            .unwrap_or(PRESETS.len()),
                        select: Box::new(|t: &mut Self, i: usize| {
                            t.interval = PRESETS[i];
                            let d = Duration::from_secs(t.interval);
                            let _ = t.control.send(Control::Interval(d));
                        }),
                        options: PRESETS
                            .iter()
                            .map(|&s| RadioItem {
                                label: every(s),
                                ..Default::default()
                            })
                            .collect(),
                    }
                    .into(),
                ],
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

pub fn run<F>(out: PathBuf, interval: Duration, watch: F) -> Result<(), String>
where
    F: FnOnce(Receiver<Control>, Notify) + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let tray = Tray {
        status: WAITING.into(),
        paused: false,
        interval: interval.as_secs(),
        control: tx,
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
            let paused = matches!(s, Status::Paused(_));
            handle.update(|t| {
                t.status = text;
                t.paused = paused;
            });
        }),
    );
    Ok(())
}

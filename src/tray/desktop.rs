//! Windows and macOS: tray-icon on a tao event loop, which must own the main
//! thread on macOS. The watch loop runs on a worker thread and wakes the event
//! loop through its proxy.

use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

use super::{Notify, WAITING, icon_rgba, label};

enum UserEvent {
    Status(String),
    Menu(MenuEvent),
}

pub fn run<F>(watch: F) -> Result<(), String>
where
    F: FnOnce(Receiver<()>, Notify) + Send + 'static,
{
    #[allow(unused_mut)]
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    #[cfg(target_os = "macos")]
    {
        // A menu bar item only: no Dock icon, no app menu.
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Menu(e));
    }));

    let status = MenuItem::new(WAITING, false, None);
    let resync = MenuItem::new("Resync now", true, None);
    let quit = MenuItem::new("Quit bwsshd", true, None);
    let menu = Menu::with_items(&[&status, &PredefinedMenuItem::separator(), &resync, &quit])
        .map_err(|e| format!("tray menu: {e}"))?;
    let icon = Icon::from_rgba(icon_rgba(32), 32, 32).map_err(|e| format!("tray icon: {e}"))?;

    let (tx, rx) = mpsc::channel();
    let proxy = event_loop.create_proxy();
    std::thread::spawn(move || {
        watch(
            rx,
            Box::new(move |s| {
                let _ = proxy.send_event(UserEvent::Status(label(&s, Instant::now())));
            }),
        )
    });

    let mut builder = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("bwsshd")
            .with_icon(icon),
    );
    let mut tray = None;
    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        match event {
            // macOS needs the loop running before the icon exists.
            Event::NewEvents(StartCause::Init) => {
                match builder.take().map(TrayIconBuilder::build) {
                    Some(Ok(t)) => tray = Some(t),
                    Some(Err(e)) => {
                        info!("tray: {e}");
                        std::process::exit(1);
                    }
                    None => {}
                }
            }
            Event::UserEvent(UserEvent::Status(text)) => {
                status.set_text(&text);
                if let Some(t) = &tray {
                    let _ = t.set_tooltip(Some(format!("bwsshd: {text}")));
                }
            }
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *resync.id() => {
                let _ = tx.send(());
            }
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *quit.id() => {
                info!("quit from the tray");
                std::process::exit(0);
            }
            _ => {}
        }
    })
}

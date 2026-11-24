//! Windows and macOS: tray-icon on a tao event loop, which must own the main
//! thread on macOS. The watch loop runs on a worker thread and wakes the event
//! loop through its proxy. Left click opens a small webview panel styled like
//! the website; right click opens the menu.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tao::dpi::{LogicalSize, PhysicalPosition};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::{Window, WindowBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, Rect, TrayIconBuilder, TrayIconEvent};
use wry::WebViewBuilder;

use super::{
    COMMIT, COMMIT_DATE, Control, Notify, PRESETS, Status, VERSION, WAITING, icon_rgba, label,
    open_config, version_line,
};

enum UserEvent {
    Status(Status),
    Menu(MenuEvent),
    Tray(TrayIconEvent),
    Panel(String),
}

const PANEL_W: f64 = 360.0;
const PANEL_H: f64 = 510.0;

pub fn run<F>(out: PathBuf, interval: Duration, watch: F) -> Result<(), String>
where
    F: FnOnce(Receiver<Control>, Notify) + Send + 'static,
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
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Tray(e));
    }));

    let status = MenuItem::new(WAITING, false, None);
    let resync = MenuItem::new("Resync now", true, None);
    let pause = MenuItem::new("Pause auto sync", true, None);
    let open = MenuItem::new("Open generated config", true, None);
    let quit = MenuItem::new("Quit bwsshd", true, None);
    let menu = Menu::with_items(&[
        &MenuItem::new(format!("bwsshd {}", version_line()), false, None),
        &status,
        &PredefinedMenuItem::separator(),
        &resync,
        &pause,
        &open,
        &quit,
    ])
    .map_err(|e| format!("tray menu: {e}"))?;
    let icon = tray_icon(false)?;

    let mut wb = WindowBuilder::new()
        .with_title("bwsshd")
        .with_inner_size(LogicalSize::new(PANEL_W, PANEL_H))
        .with_resizable(false)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_visible(false);
    #[cfg(windows)]
    {
        use tao::platform::windows::WindowBuilderExtWindows;
        wb = wb.with_skip_taskbar(true);
    }
    let window = wb
        .build(&event_loop)
        .map_err(|e| format!("tray panel: {e}"))?;
    let proxy = event_loop.create_proxy();
    let webview = WebViewBuilder::new()
        .with_html(panel_html(&out, interval))
        .with_background_color((10, 13, 20, 255))
        .with_ipc_handler(move |req| {
            let _ = proxy.send_event(UserEvent::Panel(req.body().clone()));
        })
        .build(&window)
        .map_err(|e| format!("tray panel: {e}"))?;

    let (tx, rx) = mpsc::channel();
    let proxy = event_loop.create_proxy();
    std::thread::spawn(move || {
        watch(
            rx,
            Box::new(move |s| {
                let _ = proxy.send_event(UserEvent::Status(s));
            }),
        )
    });

    let mut builder = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("bwsshd")
            .with_icon(icon),
    );
    let mut tray = None;
    let mut last_script = String::new();
    // Clicking the icon while the panel is open first blurs it (hiding it);
    // the click that follows must not reopen it.
    let mut hidden_at = Instant::now();
    let mut paused = false;
    #[cfg(windows)]
    let mut armed = false;
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
            Event::UserEvent(UserEvent::Status(s)) => {
                let text = label(&s, Instant::now());
                status.set_text(&text);
                if let Some(t) = &tray {
                    let _ = t.set_tooltip(Some(format!("bwsshd: {text}")));
                }
                let now_paused = matches!(s, Status::Paused(_));
                if now_paused != paused {
                    paused = now_paused;
                    pause.set_text(if paused {
                        "Resume auto sync"
                    } else {
                        "Pause auto sync"
                    });
                    if let (Some(t), Ok(i)) = (&tray, tray_icon(paused)) {
                        let _ = t.set_icon(Some(i));
                    }
                }
                last_script = panel_script(&s);
                let _ = webview.evaluate_script(&last_script);
            }
            Event::UserEvent(UserEvent::Tray(TrayIconEvent::Click {
                rect,
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            })) => {
                if window.is_visible() {
                    window.set_visible(false);
                } else if hidden_at.elapsed() > Duration::from_millis(300) {
                    place(&window, rect);
                    let _ = webview.evaluate_script(&last_script);
                    window.set_visible(true);
                    window.set_focus();
                    #[cfg(windows)]
                    {
                        armed = false;
                    }
                }
            }
            // On Windows, focus moving into the webview or back from another
            // app fires spurious Focused(false) events (the panel closed itself
            // after "Open config"), so it polls the foreground window instead.
            #[cfg(windows)]
            Event::MainEventsCleared if window.is_visible() => {
                if is_foreground(&window) {
                    armed = true;
                } else if armed {
                    window.set_visible(false);
                    hidden_at = Instant::now();
                }
                *flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(150));
            }
            #[cfg(not(windows))]
            Event::WindowEvent {
                event: tao::event::WindowEvent::Focused(false),
                ..
            } => {
                window.set_visible(false);
                hidden_at = Instant::now();
            }
            Event::UserEvent(UserEvent::Menu(e)) => {
                if e.id == *resync.id() {
                    let _ = tx.send(Control::Resync);
                } else if e.id == *pause.id() {
                    let _ = tx.send(Control::Pause(!paused));
                } else if e.id == *open.id() {
                    open_config(&out);
                } else if e.id == *quit.id() {
                    quit_now();
                }
            }
            Event::UserEvent(UserEvent::Panel(msg)) => match msg.as_str() {
                "resync" => {
                    let _ = tx.send(Control::Resync);
                }
                "pause" => {
                    let _ = tx.send(Control::Pause(true));
                }
                "resume" => {
                    let _ = tx.send(Control::Pause(false));
                }
                "open" => {
                    window.set_visible(false);
                    open_config(&out);
                }
                "hide" => window.set_visible(false),
                "quit" => quit_now(),
                // The page is local, but it is still input: only presets pass.
                m => {
                    if let Some(s) = m
                        .strip_prefix("every:")
                        .and_then(|s| s.parse().ok())
                        .filter(|s| PRESETS.contains(s))
                    {
                        let _ = tx.send(Control::Interval(Duration::from_secs(s)));
                    }
                }
            },
            _ => {}
        }
    })
}

#[cfg(windows)]
fn is_foreground(window: &Window) -> bool {
    use tao::platform::windows::WindowExtWindows;
    // SAFETY: GetForegroundWindow has no preconditions.
    let fg = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
    fg as isize == window.hwnd()
}

fn tray_icon(paused: bool) -> Result<Icon, String> {
    Icon::from_rgba(icon_rgba(32, paused), 32, 32).map_err(|e| format!("tray icon: {e}"))
}

fn quit_now() -> ! {
    info!("quit from the tray");
    std::process::exit(0);
}

/// Puts the panel next to the icon: above it when the icon sits in the lower
/// half of its screen (Windows taskbar), below it otherwise (macOS menu bar).
fn place(window: &Window, icon: Rect) {
    let size = window.outer_size();
    let (w, h) = (size.width as f64, size.height as f64);
    let (ix, iy) = (icon.position.x, icon.position.y);
    let contains = |m: &tao::monitor::MonitorHandle| {
        let (p, s) = (m.position(), m.size());
        ix >= p.x as f64
            && ix < p.x as f64 + s.width as f64
            && iy >= p.y as f64
            && iy < p.y as f64 + s.height as f64
    };
    let Some(mon) = window
        .available_monitors()
        .find(contains)
        .or_else(|| window.primary_monitor())
    else {
        return;
    };
    let (mx, my) = (mon.position().x as f64, mon.position().y as f64);
    let (mw, mh) = (mon.size().width as f64, mon.size().height as f64);
    let gap = 8.0 * mon.scale_factor();
    let x = (ix + icon.size.width as f64 / 2.0 - w / 2.0).clamp(mx + gap, mx + mw - w - gap);
    let y = if iy > my + mh / 2.0 {
        iy - h - gap
    } else {
        iy + icon.size.height as f64 + gap
    };
    window.set_outer_position(PhysicalPosition::new(x, y));
}

fn panel_html(out: &std::path::Path, interval: Duration) -> String {
    let b64 = |bytes: &[u8]| STANDARD.encode(bytes);
    let or_na = |s: &str| if s.is_empty() { "n/a" } else { s }.to_string();
    include_str!("panel.html")
        .replace(
            "{{INTER}}",
            &b64(include_bytes!("../../assets/fonts/inter.woff2")),
        )
        .replace(
            "{{GROTESK}}",
            &b64(include_bytes!("../../assets/fonts/space-grotesk.woff2")),
        )
        .replace(
            "{{MONO}}",
            &b64(include_bytes!("../../assets/fonts/jetbrains-mono.woff2")),
        )
        .replace("{{EVERY}}", &interval.as_secs().to_string())
        .replace("{{VERSION}}", VERSION)
        .replace("{{COMMIT}}", &or_na(COMMIT))
        .replace("{{DATE}}", &or_na(COMMIT_DATE))
        .replace("{{OUT}}", &html_escape(&out.display().to_string()))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The `update(...)` call that refreshes the panel. Rust's `{:?}` string
/// escaping is valid JavaScript.
fn panel_script(s: &Status) -> String {
    let ms = |t: &Instant| t.elapsed().as_millis().to_string();
    let (kind, error, ago) = match s {
        Status::Synced(t) => ("ok", "", ms(t)),
        Status::Absent(t) => ("absent", "", t.as_ref().map_or("null".into(), ms)),
        Status::Failed(e) => ("failed", e.as_str(), "null".into()),
        Status::Paused(t) => ("paused", "", t.as_ref().map_or("null".into(), ms)),
    };
    format!("window.update && update({kind:?}, {error:?}, {ago})")
}

use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging;

pub fn watch_foreground_windows(polling_interval: Duration, threshold: Duration) -> Receiver<String> {
    let (tx, rx) = sync_channel::<String>(1);
    thread::spawn(move || {
        let mut monitor = ChangeMonitor::new(String::new(), threshold);
        let mut focus = WindowFocus::default();
        let mut text_string: String;

        loop {
            sleep(polling_interval);
            text_string = focus.get_title();
            monitor.set(text_string);
            if let Some(title) = monitor.poll() {
                if let Err(e) = tx.send(title) {
                    panic!("failed during window handling: {e:?}")
                }
            }
        }

    });

    rx
}

struct WindowFocus {
    buffer: [u16; 128],
    title: String,
    no_handle: HWND,
}

impl WindowFocus {
    pub fn get_title(&mut self) -> String {
        let handle = unsafe { WindowsAndMessaging::GetForegroundWindow() };
        if handle == self.no_handle {
            return self.title.clone();
        }
        let length = unsafe { WindowsAndMessaging::GetWindowTextW(handle.clone(), &mut self.buffer) };
        if length > 0 {
            self.title = String::from_utf16_lossy(&self.buffer[0..length as usize]);
        }
        self.title.clone()
    }
}

impl Default for WindowFocus {
    fn default() -> Self {
        WindowFocus{
            buffer: [0; 128],
            title: String::new(),
            no_handle: HWND::default()
        }
    }
}


struct ChangeMonitor<T> {
    change_date: Instant,
    threshold: Duration,
    last_value: T,
    last_notified: T,
    notified: bool
}

impl<T> ChangeMonitor<T> where T: PartialEq + Clone {
    pub fn new(initial_value: T, threshold: Duration) -> ChangeMonitor<T> {
        ChangeMonitor {
            change_date: Instant::now(),
            threshold,
            last_value: initial_value.clone(),
            last_notified: initial_value,
            notified: false
        }
    }

    pub fn set(&mut self, value: T) {
        if self.last_value == value {
            return;
        }
        self.last_value = value;
        self.change_date = Instant::now();
        self.notified = false;
    }

    pub fn poll(&mut self) -> Option<T> {
        if self.notified || self.last_notified == self.last_value || (Instant::now() - self.change_date) < self.threshold {
            None
        } else {
            self.notified = true;
            self.last_notified = self.last_value.clone();
            Some(self.last_value.clone())
        }
    }
}
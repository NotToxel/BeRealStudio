use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use crate::pipeline::types::{LogEvent, LogLevel, ProgressEvent};

pub struct AppState {
    pub log_buffer: Arc<Mutex<Vec<LogEvent>>>,
    pub abort_flag: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            log_buffer: Arc::new(Mutex::new(Vec::new())),
            abort_flag: Arc::new(Mutex::new(false)),
        }
    }

    pub fn request_abort(&self) {
        if let Ok(mut flag) = self.abort_flag.lock() {
            *flag = true;
        }
    }

    pub fn clear_abort(&self) {
        if let Ok(mut flag) = self.abort_flag.lock() {
            *flag = false;
        }
    }

    pub fn is_aborted(&self) -> bool {
        self.abort_flag.lock().map(|f| *f).unwrap_or(false)
    }
}

// ─── Progress Emitter ─────────────────────────────────────────────────────────

pub struct ProgressEmitter {
    pub app: AppHandle,
    pub log_buffer: Arc<Mutex<Vec<LogEvent>>>,
    pub abort_flag: Arc<Mutex<bool>>,
    pub event_prefix: &'static str, // "toolkit" or "recapper"
}

impl ProgressEmitter {
    pub fn new(
        app: AppHandle,
        log_buffer: Arc<Mutex<Vec<LogEvent>>>,
        abort_flag: Arc<Mutex<bool>>,
        event_prefix: &'static str,
    ) -> Self {
        Self {
            app,
            log_buffer,
            abort_flag,
            event_prefix,
        }
    }

    pub fn emit_progress(&self, event: &ProgressEvent) {
        let event_name = format!("{}-progress", self.event_prefix);
        let _ = self.app.emit(&event_name, event);
    }

    pub fn emit_log(&self, level: LogLevel, message: impl Into<String>) {
        let msg = message.into();
        let timestamp = chrono::Utc::now().to_rfc3339();
        let event = LogEvent {
            level,
            message: msg.clone(),
            timestamp,
        };
        let event_name = format!("{}-log", self.event_prefix);
        let _ = self.app.emit(&event_name, &event);
        if let Ok(mut buf) = self.log_buffer.lock() {
            buf.push(event);
        }
    }

    pub fn info(&self, msg: impl Into<String>) {
        self.emit_log(LogLevel::Info, msg);
    }

    pub fn warn(&self, msg: impl Into<String>) {
        self.emit_log(LogLevel::Warn, msg);
    }

    pub fn error(&self, msg: impl Into<String>) {
        self.emit_log(LogLevel::Error, msg);
    }

    pub fn is_aborted(&self) -> bool {
        self.abort_flag.lock().map(|f| *f).unwrap_or(false)
    }
}

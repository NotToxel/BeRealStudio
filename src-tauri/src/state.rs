use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{AppHandle, Emitter};

use crate::pipeline::types::{LogEvent, LogLevel, ProgressEvent};

pub struct AppState {
    pub log_buffer: Arc<Mutex<Vec<LogEvent>>>,
    pub abort_flag: Arc<Mutex<bool>>,
    pub active_jobs: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            log_buffer: Arc::new(Mutex::new(Vec::new())),
            abort_flag: Arc::new(Mutex::new(false)),
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_job(&self, job_id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        if let Ok(mut map) = self.active_jobs.lock() {
            map.insert(job_id.to_string(), flag.clone());
        }
        flag
    }

    pub fn unregister_job(&self, job_id: &str) {
        if let Ok(mut map) = self.active_jobs.lock() {
            map.remove(job_id);
        }
    }

    pub fn cancel_job(&self, job_id: &str) -> bool {
        if let Ok(map) = self.active_jobs.lock() {
            if let Some(flag) = map.get(job_id) {
                flag.store(true, Ordering::SeqCst);
                return true;
            }
        }
        false
    }

    pub fn list_active_jobs(&self) -> Vec<String> {
        if let Ok(map) = self.active_jobs.lock() {
            map.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn request_abort(&self) {
        if let Ok(mut flag) = self.abort_flag.lock() {
            *flag = true;
        }
        if let Ok(map) = self.active_jobs.lock() {
            for flag in map.values() {
                flag.store(true, Ordering::SeqCst);
            }
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

#[derive(Clone)]
pub struct ProgressEmitter {
    pub app: AppHandle,
    pub log_buffer: Arc<Mutex<Vec<LogEvent>>>,
    pub abort_flag: Arc<Mutex<bool>>,
    pub job_abort_flag: Option<Arc<AtomicBool>>,
    pub job_id: Option<String>,
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
            job_abort_flag: None,
            job_id: None,
            event_prefix,
        }
    }

    pub fn with_job(
        app: AppHandle,
        log_buffer: Arc<Mutex<Vec<LogEvent>>>,
        abort_flag: Arc<Mutex<bool>>,
        job_abort_flag: Arc<AtomicBool>,
        job_id: String,
        event_prefix: &'static str,
    ) -> Self {
        Self {
            app,
            log_buffer,
            abort_flag,
            job_abort_flag: Some(job_abort_flag),
            job_id: Some(job_id),
            event_prefix,
        }
    }

    pub fn emit_progress(&self, event: &ProgressEvent) {
        let mut evt = event.clone();
        if evt.job_id.is_none() {
            evt.job_id = self.job_id.clone();
        }

        let event_name = format!("{}-progress", self.event_prefix);
        let _ = self.app.emit(&event_name, &evt);

        if let Some(ref jid) = self.job_id {
            let targeted_name = format!("job-progress-{}", jid);
            let _ = self.app.emit(&targeted_name, &evt);
        }
    }

    pub fn emit_log(&self, level: LogLevel, message: impl Into<String>) {
        let msg = message.into();
        let timestamp = chrono::Utc::now().to_rfc3339();
        let event = LogEvent {
            job_id: self.job_id.clone(),
            level,
            message: msg.clone(),
            timestamp,
        };

        let event_name = format!("{}-log", self.event_prefix);
        let _ = self.app.emit(&event_name, &event);

        if let Some(ref jid) = self.job_id {
            let targeted_name = format!("job-log-{}", jid);
            let _ = self.app.emit(&targeted_name, &event);
        }

        if let Ok(mut buf) = self.log_buffer.lock() {
            if buf.len() >= 2000 {
                let excess = buf.len() - 1999;
                buf.drain(0..excess);
            }
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
        if let Some(ref jflag) = self.job_abort_flag {
            if jflag.load(Ordering::SeqCst) {
                return true;
            }
        }
        self.abort_flag.lock().map(|f| *f).unwrap_or(false)
    }
}

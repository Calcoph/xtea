use std::{sync::RwLock, collections::HashMap, ops::DerefMut};

use log::{Record, LevelFilter, SetLoggerError};
use once_cell::sync::Lazy;

pub struct ImguiLogger {
    messages: RwLock<Lazy<HashMap<String, Vec<String>>>>
}

impl ImguiLogger {
    pub const fn new() -> ImguiLogger {
        ImguiLogger {
            messages: RwLock::new(Lazy::new(|| HashMap::new()))
        }
    }

    pub fn init(&'static self) -> Result<(), SetLoggerError> {
        log::set_logger(self)
            .map(|()| log::set_max_level(LevelFilter::Trace))
    }

    pub fn clear(&'static self) -> HashMap<String, Vec<String>> {
        let mut new_map = HashMap::new();
        let mut messages = self.messages.write().unwrap();
        let messages = messages.deref_mut().deref_mut();
        std::mem::swap(messages, &mut new_map);

        new_map
    }
}

impl log::Log for ImguiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let target = metadata.target();
        // filter out wgpu logs
        let filtered_logs = vec!["wgpu", "naga"];
        let mut is_enabled = true;
        for filter in filtered_logs {
            if target.starts_with(filter) {
                is_enabled = false
            }
        }

        is_enabled
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut messages_map = self.messages.write().unwrap();
            let new_message = format!("{} - {}", record.level(), record.args());
            let key = record.target();
            match messages_map.get_mut(key) {
                Some(msg_list) => msg_list.push(new_message),
                None => {messages_map.insert(key.to_string(), vec![new_message]);},
            };
        }
    }

    fn flush(&self) {}
}

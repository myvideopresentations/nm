use ctrlc;
use rocket::tokio::sync::broadcast::Sender;
use crate::writer::Write;
use crate::{InterfaceInfo, InterfaceStats, Message, Result};

pub struct SimpleWriter {
    writer: Sender<Message>,
    prev_stats: InterfaceStats,
    info: InterfaceInfo
}

impl SimpleWriter {
    pub fn new(
        writer: Sender<Message>,
        info: &InterfaceInfo,
        initial_stats: InterfaceStats,
    ) -> Result<SimpleWriter> {
        Ok(SimpleWriter {
            writer,
            prev_stats: initial_stats,
            info: info.clone()
        })
    }
}

impl Write for SimpleWriter {
    fn setup_shutdown(&mut self, callback: Box<dyn Fn() + 'static + Send>) -> Result<()> {
        ctrlc::set_handler(move || (*callback)()).expect("Failed to set Ctrl+C handler");
        Ok(())
    }

    fn update(&mut self, stats: InterfaceStats) -> Result<()> {
        let diff = &stats - &self.prev_stats;
        self.prev_stats = stats;
        let mess = Message {
            stats: diff,
            info: self.info.clone()
        };
        // A send 'fails' if there are no active subscribers. That's okay.
        if let Err(_) = self.writer.send(mess) {

        };

        Ok(())
    }
}

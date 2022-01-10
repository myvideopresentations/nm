use crate::{InterfaceStats, Result};

pub mod out_simple;

pub trait Write {
    fn setup_shutdown(&mut self, callback: Box<dyn Fn() + 'static + Send>) -> Result<()>;
    fn update(&mut self, stats: InterfaceStats) -> Result<()>;
}

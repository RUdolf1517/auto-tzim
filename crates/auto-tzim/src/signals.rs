use auto_tzim_core::runner::{Cancellation, Interrupt};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::{Handle, Signals};
use std::io;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct ApplySignals {
    pub cancellation: Arc<Cancellation>,
    handle: Handle,
    thread: Option<JoinHandle<()>>,
}

impl ApplySignals {
    pub fn install() -> io::Result<Self> {
        let mut signals = Signals::new([SIGINT, SIGTERM])?;
        let handle = signals.handle();
        let cancellation = Arc::new(Cancellation::default());
        let receiver = Arc::clone(&cancellation);
        let thread = thread::Builder::new()
            .name("auto-tzim-signals".into())
            .spawn(move || {
                for signal in signals.forever() {
                    match signal {
                        SIGINT => receiver.request(Interrupt::Int),
                        SIGTERM => receiver.request(Interrupt::Term),
                        _ => {}
                    }
                }
            })?;
        Ok(Self {
            cancellation,
            handle,
            thread: Some(thread),
        })
    }
}

impl Drop for ApplySignals {
    fn drop(&mut self) {
        self.handle.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

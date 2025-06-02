use std::io::{Result as IoResult, Write};
use tokio::sync::mpsc::UnboundedSender;

pub struct ChannelWriter {
    pub sender: UnboundedSender<String>,
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            for line in s.lines() {
                let _ = self.sender.send(line.to_string());
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

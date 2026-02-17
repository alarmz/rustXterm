use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};

pub struct PtySession {
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
}

pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn spawn(
        &mut self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<Box<dyn Read + Send>> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(Self::default_shell());
        cmd.env("TERM", "xterm-256color");

        let _child = pair.slave.spawn_command(cmd)?;
        // Drop the slave side - the child process owns it now
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        self.sessions.insert(
            session_id.to_string(),
            PtySession {
                master: pair.master,
                writer,
            },
        );

        Ok(reader)
    }

    pub fn write(&mut self, session_id: &str, data: &[u8]) -> anyhow::Result<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.writer.write_all(data)?;
            session.writer.flush()?;
        }
        Ok(())
    }

    pub fn resize(&mut self, session_id: &str, cols: u16, rows: u16) -> anyhow::Result<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        }
        Ok(())
    }

    pub fn close(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    fn default_shell() -> &'static str {
        if cfg!(target_os = "windows") {
            "powershell.exe"
        } else {
            "/bin/bash"
        }
    }
}

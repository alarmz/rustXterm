use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};

pub struct PtySession {
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
    pub child: Box<dyn Child + Send>,
}

#[derive(Default)]
pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
}

impl PtyManager {
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

        let child = pair.slave.spawn_command(cmd)?;
        // Drop the slave side - the child process owns it now
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        self.sessions.insert(
            session_id.to_string(),
            PtySession {
                master: pair.master,
                writer,
                child,
            },
        );

        Ok(reader)
    }

    /// Remove and return a session (used by AppSessionManager to take ownership).
    pub fn take_session(&mut self, session_id: &str) -> Option<PtySession> {
        self.sessions.remove(session_id)
    }

    fn default_shell() -> String {
        if cfg!(target_os = "windows") {
            "powershell.exe".to_string()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        }
    }
}

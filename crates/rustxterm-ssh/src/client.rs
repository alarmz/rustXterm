use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::error::SshError;

/// Bounded channel capacity for command messages (backpressure).
const CMD_CHANNEL_CAPACITY: usize = 256;

/// Default timeout for the entire connect + auth + shell open sequence.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Commands sent from the SshClient to the background channel task.
#[derive(Debug)]
enum ChannelCommand {
    Data(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    Close,
}

/// Handler for SSH client events (server key checking, etc.).
pub struct SshHandler;

#[async_trait::async_trait]
impl russh::client::Handler for SshHandler {
    type Error = SshError;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh_keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all server keys for now.
        // TODO (Phase 3): Implement known_hosts verification.
        Ok(true)
    }
}

/// An SSH client connection with an interactive shell channel.
pub struct SshClient {
    cmd_tx: mpsc::Sender<ChannelCommand>,
    handle: Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>>,
}

impl SshClient {
    /// Get a shared reference to the underlying SSH connection handle.
    ///
    /// Used to open additional channels (e.g. SFTP, port forwarding).
    /// Returns an `Arc<Mutex<Handle>>` so the handle can be used outside
    /// std::sync::Mutex guards (lock briefly for each operation).
    pub fn handle(&self) -> Arc<tokio::sync::Mutex<russh::client::Handle<SshHandler>>> {
        Arc::clone(&self.handle)
    }

    /// Connect to an SSH server and open an interactive shell.
    ///
    /// Returns the client and a receiver for data coming from the remote shell.
    /// The entire operation is wrapped in a 30-second timeout.
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Vec<u8>>), SshError> {
        tokio::time::timeout(
            CONNECT_TIMEOUT,
            Self::connect_inner(host, port, username, password, cols, rows),
        )
        .await
        .map_err(|_| SshError::Timeout)?
    }

    async fn connect_inner(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Vec<u8>>), SshError> {
        let config = Arc::new(russh::client::Config::default());
        let handler = SshHandler;

        let mut handle = Self::establish_connection(config, host, port, handler).await?;
        Self::authenticate(&mut handle, username, password).await?;
        let channel = Self::open_shell(&mut handle, cols, rows).await?;

        let (data_tx, data_rx) = mpsc::unbounded_channel();
        let (cmd_tx, cmd_rx) = mpsc::channel(CMD_CHANNEL_CAPACITY);

        // Background task: owns the channel, multiplexes reads and writes.
        Self::spawn_channel_task(channel, cmd_rx, data_tx);

        Ok((
            Self {
                cmd_tx,
                handle: Arc::new(tokio::sync::Mutex::new(handle)),
            },
            data_rx,
        ))
    }

    async fn establish_connection(
        config: Arc<russh::client::Config>,
        host: &str,
        port: u16,
        handler: SshHandler,
    ) -> Result<russh::client::Handle<SshHandler>, SshError> {
        let addr = format!("{host}:{port}");
        russh::client::connect(config, &*addr, handler)
            .await
            .map_err(|e| SshError::ConnectionFailed(e.to_string()))
    }

    async fn authenticate(
        handle: &mut russh::client::Handle<SshHandler>,
        username: &str,
        password: &str,
    ) -> Result<(), SshError> {
        let authenticated = handle
            .authenticate_password(username, password)
            .await
            .map_err(|e| SshError::AuthenticationFailed(e.to_string()))?;

        if !authenticated {
            return Err(SshError::AuthenticationFailed(
                "authentication rejected by server".to_string(),
            ));
        }
        Ok(())
    }

    async fn open_shell(
        handle: &mut russh::client::Handle<SshHandler>,
        cols: u16,
        rows: u16,
    ) -> Result<russh::Channel<russh::client::Msg>, SshError> {
        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        channel
            .request_pty(false, "xterm-256color", cols as u32, rows as u32, 0, 0, &[])
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        channel
            .request_shell(false)
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        Ok(channel)
    }

    fn spawn_channel_task(
        mut channel: russh::Channel<russh::client::Msg>,
        mut cmd_rx: mpsc::Receiver<ChannelCommand>,
        data_tx: mpsc::UnboundedSender<Vec<u8>>,
    ) {
        tokio::spawn(async move {
            // Buffer for channel.wait() so we don't lose data on cancellation.
            let mut pending_msg: Option<russh::ChannelMsg> = None;

            loop {
                // Process any pending message from a previous cancelled wait.
                if let Some(msg) = pending_msg.take() {
                    match msg {
                        russh::ChannelMsg::Data { ref data } => {
                            if data_tx.send(data.to_vec()).is_err() {
                                break;
                            }
                        }
                        russh::ChannelMsg::Eof => break,
                        _ => {}
                    }
                }

                tokio::select! {
                    biased;

                    // Prioritize processing commands from the client
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(ChannelCommand::Data(data)) => {
                                if channel.data(&data[..]).await.is_err() {
                                    break;
                                }
                            }
                            Some(ChannelCommand::Resize { cols, rows }) => {
                                let _ = channel.window_change(cols, rows, 0, 0).await;
                            }
                            Some(ChannelCommand::Close) | None => {
                                let _ = channel.eof().await;
                                break;
                            }
                        }
                    }
                    // Read data from the remote shell
                    msg = channel.wait() => {
                        match msg {
                            Some(russh::ChannelMsg::Data { ref data }) => {
                                if data_tx.send(data.to_vec()).is_err() {
                                    break;
                                }
                            }
                            Some(russh::ChannelMsg::Eof) | None => break,
                            other => {
                                // Store unhandled messages in case of cancellation
                                pending_msg = other;
                            }
                        }
                    }
                }
            }
        });
    }

    /// Send data (user input) to the remote shell.
    pub fn write(&self, data: &[u8]) -> Result<(), SshError> {
        self.cmd_tx
            .try_send(ChannelCommand::Data(data.to_vec()))
            .map_err(|_| SshError::ChannelError("channel closed or full".to_string()))
    }

    /// Resize the remote PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), SshError> {
        self.cmd_tx
            .try_send(ChannelCommand::Resize {
                cols: cols as u32,
                rows: rows as u32,
            })
            .map_err(|_| SshError::ChannelError("channel closed or full".to_string()))
    }

    /// Disconnect from the SSH server.
    pub async fn disconnect(self) -> Result<(), SshError> {
        let _ = self.cmd_tx.send(ChannelCommand::Close).await;
        let handle = self.handle.lock().await;
        handle
            .disconnect(russh::Disconnect::ByApplication, "", "en")
            .await?;
        Ok(())
    }
}

/// Send Close command on drop so SSH sessions don't linger on the server.
impl Drop for SshClient {
    fn drop(&mut self) {
        let _ = self.cmd_tx.try_send(ChannelCommand::Close);
    }
}

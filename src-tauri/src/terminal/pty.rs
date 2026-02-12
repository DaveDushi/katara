use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::Emitter;

/// Handle to a spawned PTY terminal instance.
///
/// Non-Sync PTY handles are wrapped in Mutex so the struct is Send + Sync,
/// required by AppState (behind tokio::sync::RwLock in Arc).
pub struct PtyHandle {
    pub id: String,
    writer: Mutex<Box<dyn Write + Send>>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
    master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
}

// Safety: all non-Sync fields are behind Mutex.
unsafe impl Sync for PtyHandle {}

#[derive(Clone, Serialize)]
pub struct TerminalDataPayload {
    pub id: String,
    pub data: String,
}

impl PtyHandle {
    /// Spawn a new PTY terminal.
    pub fn spawn(
        id: String,
        rows: u16,
        cols: u16,
        cwd: Option<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<Self, String> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new_default_prog();
        if let Some(ref dir) = cwd {
            cmd.cwd(dir);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;

        // Spawn a blocking reader thread that forwards PTY output to the frontend
        let pty_id = id.clone();
        tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app_handle.emit(
                            "terminal:data",
                            TerminalDataPayload {
                                id: pty_id.clone(),
                                data,
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(PtyHandle {
            id,
            writer: Mutex::new(writer),
            _child: child,
            master: Mutex::new(pair.master),
        })
    }

    /// Write data (user keystrokes) to the PTY.
    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        self.writer
            .lock()
            .map_err(|e| format!("PTY writer lock poisoned: {}", e))?
            .write_all(data)
            .map_err(|e| format!("PTY write error: {}", e))
    }

    /// Resize the PTY.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), String> {
        self.master
            .lock()
            .map_err(|e| format!("PTY master lock poisoned: {}", e))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("PTY resize error: {}", e))
    }
}

//! Opening a file dropped on the window, the same way the Open dialog does.
//!
//! winit hands a drop over in three steps: the drag enters (we accept it as a copy), it is
//! dropped (we ask for the file list), and the list arrives. Only the first local file is
//! opened; its folder becomes the playlist exactly as for a picked file.

use super::App;
use crate::controller::Engine;
use std::path::PathBuf;
use winit::data_transfer::TypeHint;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, DndAction};

/// The first regular file in a dropped list, if any.
pub(super) fn first_file(paths: Vec<PathBuf>) -> Option<PathBuf> {
    paths.into_iter().find(|p| p.is_file())
}

impl<E: Engine> App<E> {
    /// Handles the drag-and-drop events; true when the event was one of them.
    pub(super) fn drop_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: &WindowEvent,
    ) -> bool {
        match event {
            WindowEvent::DragEntered { id, .. } => {
                // Without an accepted action the OS refuses the drop.
                let _ = event_loop.set_valid_dnd_actions(*id, &[DndAction::Copy]);
            }
            WindowEvent::DragPosition { .. } | WindowEvent::DragLeft { .. } => {}
            WindowEvent::DragDropped { id, .. } => {
                if let Err(e) = event_loop.fetch_data_transfer(*id, &TypeHint::UriList) {
                    eprintln!("{{\"warning\":\"dropped data unavailable\",\"detail\":\"{e}\"}}");
                }
            }
            WindowEvent::DataTransferReceived { value, .. } => {
                match value.try_as_file_paths().map(first_file) {
                    Ok(Some(path)) => {
                        self.shared.borrow_mut().controller.open(&path);
                        self.opened();
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("{{\"warning\":\"dropped data unreadable\",\"detail\":\"{e}\"}}")
                    }
                }
            }
            _ => return false,
        }
        true
    }

    /// After a file was opened: redraw, and remount when the playlist no longer fits.
    pub(super) fn opened(&mut self) {
        let mut shared = self.shared.borrow_mut();
        shared.dirty = true;
        if !shared.fits() {
            (self.remount, self.closing) = (true, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::first_file;

    #[test]
    fn a_drop_opens_its_first_regular_file() {
        let dir = std::env::temp_dir().join(format!("kk-drop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.mkv");
        std::fs::write(&file, b"").unwrap();
        assert_eq!(first_file(vec![dir.clone(), file.clone()]), Some(file));
        assert_eq!(first_file(vec![dir.clone()]), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

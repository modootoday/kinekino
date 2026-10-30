use crate::domain::local_video::LocalVideo;
use anyhow::Result;
use std::path::PathBuf;
use windows::{
    Win32::{
        Foundation::HWND,
        System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree},
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR,
            FOS_PATHMUSTEXIST, FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH,
        },
    },
    core::w,
};
use wry::raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub struct LocalFilePicker(HWND);
impl LocalFilePicker {
    pub fn new(parent: &impl HasWindowHandle) -> Result<Self> {
        let RawWindowHandle::Win32(h) = parent.window_handle()?.as_raw() else {
            anyhow::bail!("Win32 parent required");
        };
        Ok(Self(HWND(h.hwnd.get() as *mut _)))
    }
    pub fn pick(&self) -> Result<Option<LocalVideo>> {
        // SAFETY: called on the WebView/eframe STA UI thread with its live parent HWND.
        unsafe {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(
                dialog.GetOptions()?
                    | FOS_FILEMUSTEXIST
                    | FOS_PATHMUSTEXIST
                    | FOS_FORCEFILESYSTEM
                    | FOS_NOCHANGEDIR,
            )?;
            dialog.SetTitle(w!("로컬 영상 열기"))?;
            dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
                pszName: w!("동영상"),
                pszSpec: w!("*.mp4;*.mkv;*.webm;*.mov;*.avi;*.m4v;*.ts;*.m2ts;*.mpg;*.mpeg"),
            }])?;
            if let Err(error) = dialog.Show(Some(self.0)) {
                if error.code().0 as u32 == 0x800704C7 {
                    return Ok(None);
                }
                return Err(error.into());
            }
            let raw = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let text = raw.to_string();
            CoTaskMemFree(Some(raw.0.cast()));
            let path = PathBuf::from(text?).canonicalize()?;
            anyhow::ensure!(path.is_file(), "Selected video is not a file");
            Ok(Some(LocalVideo::selected(path)?))
        }
    }
}

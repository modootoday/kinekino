use crate::{
    application::{
        api::{ApiError, ErrorCode},
        subtitles::path_picker::SubtitlePathPicker,
    },
    domain::subtitles::SubtitleFormat,
};
use windows::{
    Win32::{
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR,
            FOS_OVERWRITEPROMPT, FOS_PATHMUSTEXIST, FileOpenDialog, FileSaveDialog, IFileDialog,
            IFileOpenDialog, IFileSaveDialog, SIGDN_FILESYSPATH,
        },
    },
    core::{Interface, w},
};

pub struct NativeSubtitlePathPicker;

impl SubtitlePathPicker for NativeSubtitlePathPicker {
    fn open_subtitle(&self) -> Result<Option<String>, ApiError> {
        let _apartment = Apartment::initialize()?;
        // SAFETY: this call owns an initialized STA apartment for the dialog lifetime.
        unsafe {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(dialog_error)?;
            dialog
                .SetOptions(
                    dialog.GetOptions().map_err(dialog_error)?
                        | FOS_FILEMUSTEXIST
                        | FOS_PATHMUSTEXIST
                        | FOS_FORCEFILESYSTEM
                        | FOS_NOCHANGEDIR,
                )
                .map_err(dialog_error)?;
            dialog
                .SetTitle(w!("자막 파일 열기"))
                .map_err(dialog_error)?;
            dialog
                .SetFileTypes(&[COMDLG_FILTERSPEC {
                    pszName: w!("SRT / WebVTT"),
                    pszSpec: w!("*.srt;*.vtt"),
                }])
                .map_err(dialog_error)?;
            selected_path(&dialog.cast().map_err(dialog_error)?)
        }
    }

    fn save_subtitle(&self, format: SubtitleFormat) -> Result<Option<String>, ApiError> {
        let _apartment = Apartment::initialize()?;
        // SAFETY: all COM objects remain local to this initialized GUI-thread STA.
        unsafe {
            let dialog: IFileSaveDialog =
                CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(dialog_error)?;
            dialog
                .SetOptions(
                    (dialog.GetOptions().map_err(dialog_error)?
                        | FOS_PATHMUSTEXIST
                        | FOS_FORCEFILESYSTEM
                        | FOS_NOCHANGEDIR)
                        & !FOS_OVERWRITEPROMPT,
                )
                .map_err(dialog_error)?;
            dialog
                .SetTitle(w!("자막을 새 파일로 저장"))
                .map_err(dialog_error)?;
            let (extension, name, filter) = match format {
                SubtitleFormat::Srt => (
                    w!("srt"),
                    w!("subtitles.srt"),
                    COMDLG_FILTERSPEC {
                        pszName: w!("SRT"),
                        pszSpec: w!("*.srt"),
                    },
                ),
                SubtitleFormat::Vtt => (
                    w!("vtt"),
                    w!("subtitles.vtt"),
                    COMDLG_FILTERSPEC {
                        pszName: w!("WebVTT"),
                        pszSpec: w!("*.vtt"),
                    },
                ),
                SubtitleFormat::Ass => (
                    w!("ass"),
                    w!("subtitles.ass"),
                    COMDLG_FILTERSPEC {
                        pszName: w!("ASS"),
                        pszSpec: w!("*.ass"),
                    },
                ),
            };
            dialog
                .SetDefaultExtension(extension)
                .map_err(dialog_error)?;
            dialog.SetFileName(name).map_err(dialog_error)?;
            dialog.SetFileTypes(&[filter]).map_err(dialog_error)?;
            let selected = selected_path(&dialog.cast().map_err(dialog_error)?)?;
            if selected
                .as_ref()
                .is_some_and(|path| std::fs::symlink_metadata(path).is_ok())
            {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Output already exists; choose a new path",
                ));
            }
            Ok(selected)
        }
    }
}

unsafe fn selected_path(dialog: &IFileDialog) -> Result<Option<String>, ApiError> {
    // SAFETY: caller maintains a live STA apartment and COM dialog on this thread.
    unsafe {
        if let Err(error) = dialog.Show(None) {
            if error.code().0 as u32 == 0x800704C7 {
                return Ok(None);
            }
            return Err(dialog_error(error));
        }
        let raw = dialog
            .GetResult()
            .map_err(dialog_error)?
            .GetDisplayName(SIGDN_FILESYSPATH)
            .map_err(dialog_error)?;
        let value = raw.to_string();
        CoTaskMemFree(Some(raw.0.cast()));
        value.map(Some).map_err(dialog_error)
    }
}

struct Apartment;
impl Apartment {
    fn initialize() -> Result<Self, ApiError> {
        // SAFETY: balanced with CoUninitialize, including S_FALSE nested initialization.
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(dialog_error)?;
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
fn dialog_error<E>(_: E) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Native subtitle path dialog failed; enter a path as text",
    )
}

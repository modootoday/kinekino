use eframe::egui::{self, Color32, Rect, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Play,
    Brand,
    StartMark,
    Pause,
    Previous,
    Next,
    Back,
    Forward,
    Folder,
    FileMask,
    FileImage,
    FileDepth,
    FileJson,
    Search,
    ZoomIn,
    MoveView,
    List,
    Fullscreen,
    Volume,
    VolumeZero,
    TrackVideo,
    TrackSubtitle,
    TrackSubtitleMedia,
    TrackAudio,
    Refresh,
    Merge,
    Copy,
    Delete,
    Undo,
    Redo,
    Minimize,
    Maximize,
    Restore,
    Close,
    Segments,
    Subtitles,
    Subject,
    SelectionPointer,
    Clip,
    Image,
    Cards,
    Layers,
    Cube,
    Model,
    Microphone,
    Music,
    TextDocument,
    FileOutput,
    ScanText,
    Export,
    BrushAdd,
    MaskExclude,
    Eraser,
    Hand,
    Eye,
    EyeOff,
    Lock,
    Unlock,
    Grip,
    MoreVertical,
    Plus,
    Minus,
    Warning,
    Info,
    ChevronDown,
}

impl Icon {
    fn source(self) -> (&'static str, &'static [u8]) {
        macro_rules! asset {
            ($name:literal) => {
                (
                    concat!("bytes://player/icons/", $name, ".svg"),
                    include_bytes!(concat!("../../../../assets/design/icons/", $name, ".svg"))
                        .as_slice(),
                )
            };
        }
        macro_rules! caption {
            ($name:literal) => {
                (
                    concat!("bytes://player/window/", $name, ".svg"),
                    include_bytes!(concat!("../../../../assets/design/window/", $name, ".svg"))
                        .as_slice(),
                )
            };
        }
        match self {
            Self::Play => asset!("play"),
            Self::Brand => asset!("brand"),
            Self::StartMark => asset!("start-mark"),
            Self::Pause => asset!("pause"),
            Self::Previous => asset!("previous"),
            Self::Next => asset!("next"),
            Self::Back => asset!("back"),
            Self::Forward => asset!("forward"),
            Self::Folder => asset!("folder"),
            Self::FileMask => asset!("file-mask"),
            Self::FileImage => asset!("file-image"),
            Self::FileDepth => asset!("file-depth"),
            Self::FileJson => asset!("file-json"),
            Self::Search => asset!("search"),
            Self::ZoomIn => asset!("zoom-in"),
            Self::MoveView => asset!("move-view"),
            Self::List => asset!("list"),
            Self::Fullscreen => asset!("fullscreen"),
            Self::Volume => asset!("volume"),
            Self::VolumeZero => asset!("volume-zero"),
            Self::TrackVideo => asset!("track-video"),
            Self::TrackSubtitle => asset!("track-subtitle"),
            Self::TrackSubtitleMedia => asset!("track-subtitle-media"),
            Self::TrackAudio => asset!("track-audio"),
            Self::Refresh => asset!("refresh"),
            Self::Merge => asset!("merge"),
            Self::Copy => asset!("copy"),
            Self::Delete => asset!("delete"),
            Self::Undo => asset!("undo"),
            Self::Redo => asset!("redo"),
            Self::Segments => asset!("segments"),
            Self::Subtitles => asset!("subtitles"),
            Self::Subject => asset!("subject"),
            Self::SelectionPointer => asset!("selection-pointer"),
            Self::Clip => asset!("clip"),
            Self::Image => asset!("image"),
            Self::Cards => asset!("cards"),
            Self::Layers => asset!("layers"),
            Self::Cube => asset!("cube"),
            Self::Model => asset!("model"),
            Self::Microphone => asset!("microphone"),
            Self::Music => asset!("music"),
            Self::TextDocument => asset!("text-document"),
            Self::FileOutput => asset!("file-output"),
            Self::ScanText => asset!("scan-text"),
            Self::Export => asset!("export"),
            Self::BrushAdd => asset!("brush-add"),
            Self::MaskExclude => asset!("mask-exclude"),
            Self::Eraser => asset!("eraser"),
            Self::Hand => asset!("hand"),
            Self::Eye => asset!("eye"),
            Self::EyeOff => asset!("eye-off"),
            Self::Lock => asset!("lock"),
            Self::Unlock => asset!("unlock"),
            Self::Grip => asset!("grip"),
            Self::MoreVertical => asset!("more-vertical"),
            Self::Plus => asset!("plus"),
            Self::Minus => asset!("minus"),
            Self::Warning => asset!("warning"),
            Self::Info => asset!("info"),
            Self::ChevronDown => asset!("chevron-down"),
            Self::Minimize => caption!("minimize"),
            Self::Maximize => caption!("maximize"),
            Self::Restore => caption!("restore"),
            Self::Close => caption!("close"),
        }
    }

    pub fn image(self) -> egui::Image<'static> {
        let (uri, bytes) = self.source();
        egui::Image::from_bytes(uri, bytes)
    }

    pub fn paint(self, ui: &Ui, rect: Rect, color: Color32) {
        self.image().tint(color).paint_at(ui, rect);
    }
}

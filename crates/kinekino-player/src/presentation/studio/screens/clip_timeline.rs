use crate::{
    application::workspace_read::{ReadOrigin, WorkspaceSnapshot},
    presentation::{
        components::{
            filmstrip::{self, Region},
            timeline_ruler,
            timeline_track_header::{self, TrackKind},
        },
        design_tokens::{border, color, proportion, radius, size, space, text},
    },
};
use eframe::egui;

pub(super) fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot) {
    let end = super::view::visible_end(snapshot);
    let sample = snapshot.origin == ReadOrigin::DesignSample && snapshot.source.is_some();
    ui.weak(if sample {
        "디자인 트랙 예시 · 반복 프레임·수동 파형 · 실제 미디어 분석 결과 아님"
    } else {
        "트랙 데이터 연결 필요"
    });
    let height = size::TIMELINE_RULER
        + size::VIDEO_TRACK
        + size::CONTROL
        + size::AUDIO_TRACK
        + space::TIGHT * 2.;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let tracks =
        egui::Rect::from_min_max(rect.min + egui::vec2(size::TIMELINE_LABEL, 0.), rect.max);
    let ruler =
        egui::Rect::from_min_size(tracks.min, egui::vec2(tracks.width(), size::TIMELINE_RULER));
    timeline_ruler::paint(ui, ruler, end);
    let video = egui::Rect::from_min_size(
        ruler.left_bottom(),
        egui::vec2(tracks.width(), size::VIDEO_TRACK),
    );
    let subtitle = egui::Rect::from_min_size(
        video.left_bottom() + egui::vec2(0., space::TIGHT),
        egui::vec2(tracks.width(), size::CONTROL),
    );
    let audio = egui::Rect::from_min_size(
        subtitle.left_bottom() + egui::vec2(0., space::TIGHT),
        egui::vec2(tracks.width(), size::AUDIO_TRACK),
    );
    for (kind, row) in [
        (TrackKind::Video, video),
        (TrackKind::Subtitle, subtitle),
        (TrackKind::Audio, audio),
    ] {
        timeline_track_header::paint(
            ui,
            egui::Rect::from_min_max(egui::pos2(rect.left(), row.top()), row.left_bottom()),
            kind,
        );
        ui.painter().rect_filled(row, radius::SMALL, color::PANEL);
    }
    let regions: Vec<_> = snapshot
        .timeline
        .iter()
        .flat_map(|track| &track.segments)
        .map(|segment| Region {
            label: &segment.label,
            start: segment.in_us.0,
            end: segment.out_us.0,
        })
        .collect();
    let asset = snapshot
        .preview
        .frame
        .as_ref()
        .filter(|_| sample)
        .map(|a| a.asset_id.as_str());
    filmstrip::paint(ui, video, asset, end, &regions);
    if sample {
        for region in &regions {
            let bounds = egui::Rect::from_min_max(
                egui::pos2(
                    timeline_ruler::x(subtitle, region.start, end),
                    subtitle.top(),
                ),
                egui::pos2(
                    timeline_ruler::x(subtitle, region.end, end),
                    subtitle.bottom(),
                ),
            );
            let painter = ui
                .painter()
                .with_clip_rect(bounds.intersect(ui.clip_rect()));
            painter.rect_filled(
                bounds.shrink(border::THIN),
                radius::CONTROL,
                color::SUBTITLE_TRACK.gamma_multiply(proportion::TRACK_FILL_OPACITY),
            );
            painter.rect_stroke(
                bounds.shrink(border::THIN),
                radius::CONTROL,
                egui::Stroke::new(border::THIN, color::SUBTITLE_TRACK),
                egui::StrokeKind::Inside,
            );
            let label = painter.layout_no_wrap(
                region.label.into(),
                egui::FontId::proportional(text::SMALL),
                color::TEXT,
            );
            painter.galley(bounds.center() - label.size() / 2., label, color::TEXT);
        }
        let painter = ui.painter().with_clip_rect(audio.intersect(ui.clip_rect()));
        let amplitudes = super::view::sample_amplitudes();
        for (index, amplitude) in amplitudes.iter().enumerate() {
            let x = audio.left() + (index as f32 + 0.5) * audio.width() / amplitudes.len() as f32;
            let extent = amplitude * audio.height() / 2.;
            painter.line_segment(
                [
                    egui::pos2(x, audio.center().y - extent),
                    egui::pos2(x, audio.center().y + extent),
                ],
                egui::Stroke::new(border::THIN, color::AUDIO_WAVEFORM),
            );
        }
    }
    timeline_ruler::playhead(ui, tracks, end, snapshot.preview.position_us.0);
}

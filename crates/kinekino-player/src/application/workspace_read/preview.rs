use super::*;
use crate::domain::api_values::DecimalU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewScenario {
    Empty,
    Populated,
    LongLabels,
    NeedsInput,
}
pub struct PreviewWorkspaceProvider {
    scenario: PreviewScenario,
}
impl PreviewWorkspaceProvider {
    pub fn new(scenario: PreviewScenario) -> Self {
        Self { scenario }
    }
}
impl WorkspaceReadPort for PreviewWorkspaceProvider {
    fn snapshot(&self, workspace: WorkspaceId) -> WorkspaceSnapshot {
        let empty = self.scenario == PreviewScenario::Empty;
        let needs = self.scenario == PreviewScenario::NeedsInput;
        let state = if empty {
            ReadState::Unknown
        } else if needs {
            ReadState::NeedsInput
        } else {
            ReadState::Ready
        };
        let title = if self.scenario == PreviewScenario::LongLabels {
            "길이가 긴 작품 이름과 여러 언어가 포함된 장면 제목 — 日本語の長い題名と English demonstration label"
        } else {
            "디자인 검토용 예시 영상"
        };
        let mut snapshot = WorkspaceSnapshot {
            workspace_id: workspace,
            origin: ReadOrigin::DesignSample,
            state,
            source: (!empty).then(|| SourceSnapshot {
                source_id: "sample-source-001".into(),
                title: title.into(),
                duration_us: Some(DecimalU64(1_420_000_000)),
            }),
            preview: if empty {
                PreviewSnapshot::default()
            } else {
                PreviewSnapshot {
                    position_us: DecimalU64(52_000_000),
                    duration_us: Some(DecimalU64(1_420_000_000)),
                    frame: Some(preview_asset(workspace)),
                }
            },
            timeline: Vec::new(),
            canvas: super::preview_canvas::canvas(workspace, self.scenario),
            mask: super::preview_mask::mask(workspace, self.scenario),
            table: super::preview_tables::table(
                workspace,
                state,
                empty,
                self.scenario == PreviewScenario::LongLabels,
            ),
            inspector: Vec::new(),
            queue: Vec::new(),
            notice: Some(
                "DesignSample — 화면 검토용 예시 데이터. 실제 작업·설치·저장 상태가 아닙니다."
                    .into(),
            ),
        };
        if !empty {
            snapshot.timeline = vec![TimelineTrack {
                track_id: "sample-track-001".into(),
                label: "예시 시간 구간".into(),
                segments: vec![
                    TimelineSegment {
                        segment_id: "sample-segment-001".into(),
                        in_us: DecimalU64(0),
                        out_us: DecimalU64(15_000_000),
                        label: "검토 구간 A".into(),
                        state,
                    },
                    TimelineSegment {
                        segment_id: "sample-segment-002".into(),
                        in_us: DecimalU64(25_000_000),
                        out_us: DecimalU64(40_000_000),
                        label: "검토 구간 B".into(),
                        state,
                    },
                    TimelineSegment {
                        segment_id: "sample-segment-003".into(),
                        in_us: DecimalU64(50_000_000),
                        out_us: DecimalU64(65_000_000),
                        label: "검토 구간 C".into(),
                        state,
                    },
                ],
            }];
            snapshot.inspector = super::preview_inspector::fields(workspace, state);
            snapshot.queue = vec![QueueItem {
                item_id: "sample-queue-001".into(),
                title: "디자인 검토용 작업 예시".into(),
                state: if needs {
                    QueueState::NeedsInput
                } else {
                    QueueState::Running
                },
                progress_percent: (!needs).then_some(35),
            }];
        }
        snapshot
    }
}

fn preview_asset(workspace: WorkspaceId) -> AssetReference {
    let (asset_id, path) = match workspace {
        WorkspaceId::Obj | WorkspaceId::Proj | WorkspaceId::Clip => {
            ("demo-anime-forest-v1", "embedded://demo/anime/forest-v1")
        }
        WorkspaceId::A2d => (
            "demo-anime-traveler-v2",
            "embedded://demo/anime/traveler-v2",
        ),
        _ => ("demo-anime-coast-v1", "embedded://demo/anime/coast-v1"),
    };
    AssetReference {
        asset_id: asset_id.into(),
        path: path.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn every_workspace_and_scenario_is_explicit_and_structurally_linked() {
        for scenario in [
            PreviewScenario::Empty,
            PreviewScenario::Populated,
            PreviewScenario::LongLabels,
            PreviewScenario::NeedsInput,
        ] {
            let provider = PreviewWorkspaceProvider::new(scenario);
            for id in WorkspaceId::ALL {
                let snapshot = provider.snapshot(id);
                assert_eq!(snapshot.origin, ReadOrigin::DesignSample);
                assert_eq!(snapshot.workspace_id, id);
                assert!(snapshot.notice.is_some());
                let columns: BTreeSet<_> = snapshot
                    .table
                    .columns
                    .iter()
                    .map(|column| &column.column_id)
                    .collect();
                let rows: BTreeSet<_> = snapshot.table.rows.iter().map(|row| &row.row_id).collect();
                assert_eq!(rows.len(), snapshot.table.rows.len());
                for row in &snapshot.table.rows {
                    for cell in &row.cells {
                        assert!(columns.contains(&cell.column_id));
                    }
                }
                for track in &snapshot.timeline {
                    for segment in &track.segments {
                        assert!(segment.in_us < segment.out_us);
                    }
                }
                if scenario == PreviewScenario::Empty {
                    assert!(
                        snapshot.source.is_none()
                            && snapshot.table.rows.is_empty()
                            && snapshot.queue.is_empty()
                    );
                }
            }
        }
    }
    #[test]
    fn live_missing_adapter_does_not_return_empty_ready_success() {
        for id in WorkspaceId::ALL {
            let snapshot = LiveWorkspaceProvider.snapshot(id);
            assert_eq!(snapshot.origin, ReadOrigin::Live);
            assert_eq!(snapshot.state, ReadState::Unsupported);
            assert!(snapshot.source.is_none());
            assert!(snapshot.queue.is_empty());
        }
    }
}

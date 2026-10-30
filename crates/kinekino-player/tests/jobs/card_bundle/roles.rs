use super::*;
use kinekino_player::application::api::CardClient;
use kinekino_player::{
    application::card_bundle::BundleTemplatePin, domain::card_document::CardRole,
};

pub(super) struct RoleScenario<'a> {
    pub fixture: &'a Fixture,
    pub client: &'a dyn ApplicationClient,
    pub endpoint: &'a str,
    pub deadline: Deadline,
    pub document: &'a CardDocument,
    pub frames: &'a [SelectedCardFrame],
}
impl RoleScenario<'_> {
    pub fn execute(&self, templates: &[BundleTemplatePin]) -> Result<(), Box<CliFailure>> {
        let variants: Vec<_> = templates
            .iter()
            .filter(|template| template.role_layouts_and_pages)
            .collect();
        assert_eq!(variants.len(), 6);
        for (index, template) in variants.into_iter().enumerate() {
            let mut document = self.document.clone();
            document.document_id = format!("cli-role-project-{index}");
            document.revision = DecimalU64(1);
            document.layout.canvas = template.canvas.clone();
            let first = document.layout.cards[0].clone();
            document.layout.cards = [CardRole::Cover, CardRole::Body, CardRole::Closing]
                .into_iter()
                .enumerate()
                .map(|(position, role)| {
                    let mut card = first.clone();
                    card.card_id = format!("role-{position}");
                    card.role = role;
                    if !template.frame_required {
                        card.frame = None;
                    }
                    card
                })
                .collect();
            let saved = self
                .client
                .create_card_project(RequestContext::default(), document)
                .unwrap();
            let input = CardBundleRequest {
                project: SavedCardProjectPin {
                    document_id: saved.publication.document.document_id,
                    publication_revision: saved.publication.publication_revision,
                    publication_sha256: saved.publication_sha256,
                },
                frames: if template.frame_required {
                    self.frames.to_vec()
                } else {
                    Vec::new()
                },
                output_directory: self
                    .fixture
                    .root
                    .join(format!("role-output-{index}"))
                    .to_str()
                    .unwrap()
                    .into(),
                review_artifact_only: true,
            };
            let mut submit = request(
                "jobs.submit",
                serde_json::to_value(SubmitJob {
                    operation: "production.export_cards".into(),
                    parameters: serde_json::to_value(&input).unwrap(),
                    expected_source_sha256: None,
                    execution_timeout_ms: DecimalU64(30000),
                })
                .unwrap(),
            );
            submit.context.idempotency_key = Some(format!("role-once-{index}"));
            let admitted = self.fixture.try_call_until(
                &format!("role-submit-{index}"),
                &submit,
                self.endpoint,
                self.deadline,
                Duration::from_secs(10),
                0,
            )?;
            let id: JobId =
                serde_json::from_value(admitted["data"]["job"]["job_id"].clone()).unwrap();
            let finished = try_terminal_until(
                self.endpoint,
                &id,
                self.deadline,
                Duration::from_secs(25),
                &self.fixture.root,
            )?;
            assert_eq!(
                finished.state,
                JobState::Completed,
                "{:?}",
                finished.failure
            );
            let output = finished.output.unwrap();
            let records = manifest(&output.path, &output.sha256, output.bytes.0);
            assert_eq!(records["template"]["role_layouts_and_pages"], true);
            assert_eq!(
                records["source_frames"].as_array().unwrap().len(),
                3 * usize::from(template.frame_required)
            );
            assert_eq!(
                records["files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|entry| entry["name"].as_str().unwrap().starts_with("frame-"))
                    .count(),
                usize::from(template.frame_required),
                "Source mappings and unique bundled files are distinct"
            );
            let directory = std::path::Path::new(&output.path).parent().unwrap();
            let mut layout_hashes = BTreeSet::new();
            for (position, role) in ["cover", "body", "closing"].into_iter().enumerate() {
                let stem = format!("{:03}-{role}", position + 1);
                let pixels = fs::read(directory.join(format!("{stem}.png"))).unwrap();
                let mut decoder = png::Decoder::new(std::io::Cursor::new(pixels))
                    .read_info()
                    .unwrap();
                assert_eq!(
                    (decoder.info().width, decoder.info().height),
                    (template.canvas.width, template.canvas.height)
                );
                let mut rgba = vec![0; decoder.output_buffer_size().unwrap()];
                decoder.next_frame(&mut rgba).unwrap();
                decoder.finish().unwrap();
                let provenance: Value = serde_json::from_slice(
                    &fs::read(directory.join(format!("{stem}.provenance.json"))).unwrap(),
                )
                .unwrap();
                assert_eq!(provenance["page"]["current"], position + 1);
                assert_eq!(provenance["page"]["total"], 3);
                assert_eq!(provenance["card_role"], role);
                assert_eq!(provenance["quality_approved"], false);
                assert_eq!(
                    provenance["frame_sha256"].is_null(),
                    !template.frame_required
                );
                assert!(
                    provenance["page_layout"]
                        .as_str()
                        .unwrap()
                        .contains("SlotObservation")
                );
                layout_hashes.insert(
                    provenance["template_dto_sha256"]
                        .as_str()
                        .unwrap()
                        .to_owned(),
                );
            }
            assert_eq!(
                layout_hashes.len(),
                3,
                "Each role has a distinct declared layout"
            );
        }
        Ok(())
    }
}

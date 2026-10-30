//! Synthetic authenticated-review port contract only; never a real user quality approval.
use super::*;
struct Review {
    record: AssetBlobPin,
    mask: AssetMaskPin,
    usable: bool,
}
impl AssetReviewLease for Review {
    fn record(&self) -> &AssetBlobPin {
        &self.record
    }
    fn mask(&self) -> &AssetMaskPin {
        &self.mask
    }
    fn usable(&self) -> bool {
        self.usable
    }
}
struct Ports<'a> {
    files: &'a FixedAssetInputs,
    record: AssetBlobPin,
    mask: AssetMaskPin,
    usable: bool,
}
impl AssetInputAdmission for Ports<'_> {
    type Image = NativeAssetImage;
    type Mask = NativeAssetMask;
    type Review = Review;
    fn open_image(
        &self,
        pin: &AssetImagePin,
        control: &dyn MutationControl,
    ) -> Result<Self::Image, ApiError> {
        self.files.open_image(pin, control)
    }
    fn open_mask(
        &self,
        pin: &AssetMaskPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Mask, ApiError> {
        self.files.open_mask(pin, control)
    }
    fn open_review(
        &self,
        _record: &AssetBlobPin,
        _mask: &AssetMaskPin,
        _control: &dyn MutationControl,
    ) -> Result<Review, ApiError> {
        Ok(Review {
            record: self.record.clone(),
            mask: self.mask.clone(),
            usable: self.usable,
        })
    }
}
#[test]
fn usable_flag_without_exact_record_and_original_mask_binding_cannot_admit_review() {
    let mut fixture = Fixture::new(false, false, false, false);
    let record = AssetBlobPin {
        artifact_id: "synthetic-review".into(),
        revision: DecimalU64(2),
        sha256: "a".repeat(64),
    };
    fixture.request.review = AssetMaskReview::Reference {
        record: record.clone(),
    };
    let mut ports = Ports {
        files: &fixture.ports,
        record,
        mask: fixture.request.mask.clone(),
        usable: true,
    };
    let admitted = asset_generation::prepare(&fixture.request, &ports, &Control).unwrap();
    assert!(!admitted.unreviewed_mask_warning());
    drop(admitted);
    ports.record.revision.0 += 1;
    assert_eq!(
        asset_generation::prepare(&fixture.request, &ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    ports.record.revision.0 -= 1;
    ports.mask.source.blob.revision.0 += 1;
    assert_eq!(
        asset_generation::prepare(&fixture.request, &ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    ports.mask = fixture.request.mask.clone();
    ports.usable = false;
    assert_eq!(
        asset_generation::prepare(&fixture.request, &ports, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
}

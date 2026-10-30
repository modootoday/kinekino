//! 명시 선택 실제 표본. 역사 모델 시간축·품질 승인을 부여하지 않는다.
use super::*;
const PACKAGE: &str = "b1e2be1056e044cd3a8f9e41d16d768b6292de81c2467890d79eee844268e390";
fn source_root() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("REVIEW_MATROSKA_PACKAGE_ROOT").expect("explicit prepared actual source"),
    );
    assert!(root.is_absolute());
    root
}
pub(super) fn package_path() -> PathBuf {
    source_root()
        .join("packages")
        .join(format!("{PACKAGE}.json"))
}
pub(super) fn nomination(sha: &str) -> Value {
    assert_eq!(sha, PACKAGE);
    json!({"case":{"package_sha256":PACKAGE,"item_id":"gemma270-highlights","source_request_sha256":"658e5fbded1a1203472c937b8b7188c64915859446cb6c4e9b827d877b0d8093"},"media":{"asset_id":"explicit-sintel-source","sha256":"f12c070e295b38cfc94ebd61ac3357c3bac82d1015985f1e8da93a4c6496c46d"}})
}
pub(super) fn configure(value: &mut Value) {
    value["source_root"] = json!(source_root());
    value["video_start_us"] = json!(107_000_000);
    value["video_end_us"] = json!(134_000_000);
    value["pcm_start_pts"] = json!(1_716_000);
    value["pcm_end_pts"] = json!(2_140_800);
    value["start_us"] = json!(107_250_000);
    value["end_us"] = json!(133_800_000);
    value["max_source_bytes"] = json!(1_073_741_824u64);
    value["max_package_bytes"] = json!(2_147_483_648u64);
    value["production_total_ms"] = json!(120_000);
    value["transport_timeout_ms"] = json!(30_000);
}

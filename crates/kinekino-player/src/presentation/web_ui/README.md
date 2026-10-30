# Product Web UI source

React/TSX files are product UI declarations consumed by Meari's Rust source graph and Web document pipeline. They are not a separately implemented JavaScript application. Product policies and commands remain in the common Rust SDK.

`components/` owns reusable source components; `viewing/` and `studio/` compose them. `styles/tokens.css` is the explicit Web projection of existing native design tokens; component and screen CSS refer to those variables. Existing SVG and bundled Pretendard/OFL assets remain shared. Automated cross-target token export is still pending. P1a Native SDK/frame tests consume the existing product style and verified asset resolver; that result does not prove every declared screen or a Web build.

The first source set models the approved start screen, shared caption controls and transport. Product identity, connection status and observed time are typed props. Unconnected commands are disabled with explicit accessible reasons. Source graph composition, widget/runtime/asset constraints are tracked in `.spec/plans/player/presentation/meari-consumer-gaps.md`; library gaps are assigned to Meari rather than flattened away here.

No npm runtime, build, browser acceptance or visual parity has been claimed. The generated Web application must use Meari's actual parser/generator/builder/mount flow. Native window commands require an explicit host capability and cannot be treated as arbitrary browser window control.

`routing/routes.json` is a catalog, not runtime registration: it declares three viewing routes and twenty studio routes. Five studio entries currently name workspace source; fifteen have no workspace source. Proposal panels are SDK document editors and do not replace full card, clip or subtitle workspaces. The current scope comparison is maintained in `.spec/sots/verification/presentation/web-ui-coverage-2026-10-20.md`.

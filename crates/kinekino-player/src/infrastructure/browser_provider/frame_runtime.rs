//! Fixed DOM projection only. Application decisions and validation remain in Rust.
use super::{
    capture::{Callback, now_ms},
    error,
    navigation_events::EventSink,
};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_provider::{FrameDocumentIdentity, FrameRuntimeRequest, MAX_RUNTIME_ENVELOPE_BYTES},
};
use std::rc::Rc;
use webview2_com::{ExecuteScriptCompletedHandler, Microsoft::Web::WebView2::Win32::*};
use windows::core::HSTRING;

pub(super) type CurrentFrame = Rc<dyn Fn(&FrameDocumentIdentity) -> bool>;

pub(super) fn observe(
    frame: ICoreWebView2Frame2,
    request: FrameRuntimeRequest,
    current: CurrentFrame,
    sink: EventSink,
) -> Result<(), ApiError> {
    if !current(request.identity()) {
        return Err(error(
            ErrorCode::RevisionConflict,
            "Child runtime document is no longer current",
        ));
    }
    let handler = ExecuteScriptCompletedHandler::create(Box::new(move |result, json| {
        if !current(request.identity()) {
            return Ok(());
        }
        if result.is_err() {
            sink(Callback::Failed(error(
                ErrorCode::EngineFailed,
                "Child runtime observation failed",
            )));
        } else if json.len() > MAX_RUNTIME_ENVELOPE_BYTES {
            sink(Callback::Failed(error(
                ErrorCode::ResourceExhausted,
                "Child runtime envelope exceeds limit",
            )));
        } else {
            sink(Callback::FrameRuntime(request, json, now_ms()));
        }
        Ok(())
    }));
    // SAFETY: fixed read-only projection on the actual current frame's creating STA.
    unsafe { frame.ExecuteScript(&HSTRING::from(PROJECTION), &handler) }.map_err(|_| {
        error(
            ErrorCode::EngineFailed,
            "Child runtime observation could not start",
        )
    })
}

// Minimal WebView DOM serialization adapter, tracked for Rust/engine-interface minimization.
// No player method calls, event injection, CSS mutation, network calls or caller-supplied script.
const PROJECTION: &str = r#"(()=>{
const visibility=e=>{if(!e)return {present:false,displayed:false,visible:false,has_layout:false,center_in_viewport:false,center_hit:false};
const s=getComputedStyle(e),r=e.getBoundingClientRect(),x=r.left+r.width/2,y=r.top+r.height/2;
const inside=x>=0&&y>=0&&x<innerWidth&&y<innerHeight;
const hit=inside?document.elementFromPoint(x,y):null;
return {present:true,displayed:s.display!=='none',visible:s.visibility==='visible'&&Number(s.opacity)>0,has_layout:r.width>0&&r.height>0,center_in_viewport:inside,center_hit:!!hit&&(e===hit||e.contains(hit))};};
const nodes=document.querySelectorAll('video');
const finite=v=>Number.isFinite(v)&&v>=0?v:null;
return {document_url:location.href.slice(0,8193),document_ready_state:document.readyState,document_visibility:document.visibilityState,
fire_player_defined:typeof window.FirePlayer==='function',fireload_defined:typeof window.fireload==='function',jwplayer_defined:typeof window.jwplayer==='function',player_loaded_flag:typeof window.player_loaded==='boolean'?window.player_loaded:null,
play_overlay:visibility(document.querySelector('.play-button-outer')),player_root:visibility(document.getElementById('playerbase')),popup_overlay:visibility(document.querySelector('.pppx')),
video_count:nodes.length,videos:Array.prototype.slice.call(nodes,0,8).map(v=>({ready_state:v.readyState,network_state:v.networkState,paused:v.paused,ended:v.ended,muted:v.muted,current_time_seconds:finite(v.currentTime),duration_seconds:finite(v.duration),error_code:v.error?v.error.code:null,current_src:v.currentSrc.slice(0,8193),video_width:v.videoWidth,video_height:v.videoHeight}))};
})()"#;

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

// This module contains traits in layout used generically
//   in the rest of Servo.
// The traits are here instead of in layout so
//   that these modules won't have to depend on layout.

use std::rc::Rc;
use std::{borrow::Cow, sync::atomic::AtomicBool};
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use gfx::font_cache_thread::{self, FontCacheThread};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use metrics::PaintTimeMetrics;
use msg::constellation_msg::{
    BackgroundHangMonitorRegister, PipelineId, TopLevelBrowsingContextId,
};
use net_traits::image_cache::ImageCache;
use profile_traits::{mem, time};
use script_traits::{
    ConstellationControlMsg, InitialScriptState, LayoutControlMsg, LayoutMsg as ConstellationMsg, LoadData, WebrenderIpcSender, WindowSizeData
};
use servo_url::ServoUrl;


pub struct LayoutConfig {
    pub id: PipelineId,
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,
    pub url: ServoUrl,
    pub is_iframe: bool,
    pub constellation_chan: IpcSender<ConstellationMsg>,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub image_cache: Arc<dyn ImageCache>,
    pub font_cache_thread: FontCacheThread,
    pub time_profiler_chan: time::ProfilerChan,
    pub mem_profiler_chan: mem::ProfilerChan,
    pub webrender_api_sender: WebrenderIpcSender,
    pub paint_time_metrics: PaintTimeMetrics,
    pub busy: Arc<AtomicBool>,
    pub window_size: WindowSizeData,
}

/// The initial data required to create a new layout attached to an existing script thread.
pub struct LayoutChildConfig {
    pub id: PipelineId,
    pub url: ServoUrl,
    pub is_parent: bool,
    pub constellation_chan: IpcSender<ConstellationMsg>,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub image_cache: Arc<dyn ImageCache>,
    pub paint_time_metrics: PaintTimeMetrics,
    pub layout_is_busy: Arc<AtomicBool>,
    pub window_size: WindowSizeData,
}
pub trait LayoutFactory: Send + Sync {
    fn create(&self, config: LayoutConfig) -> Box<dyn Layout>;
}

/// This trait allows creating a `ScriptThread` without depending on the `script`
/// crate.
pub trait ScriptThreadFactory {
    /// Type of message sent from script to layout.
    type Message;
    /// Create a `ScriptThread`.
    fn create(
        state: InitialScriptState,
        layout_factory: Arc<dyn LayoutFactory>,
        font_cache_thread: FontCacheThread,
        load_data: LoadData,
        user_agent: Cow<'static, str>,
    ) -> (Sender<Self::Message>, Receiver<Self::Message>);
}

pub trait Layout {
    fn process(&mut self, msg: script_layout_interface::message::Msg);
    fn handle_constellation_msg(&mut self, msg: script_traits::LayoutControlMsg);
    fn handle_font_cache_msg(&mut self);
    fn create_new_layout(&self, init: LayoutChildConfig) -> Box<dyn Layout>;
    fn rpc(&self) -> Box<dyn script_layout_interface::rpc::LayoutRPC>;
}
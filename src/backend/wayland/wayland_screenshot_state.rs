use crate::backend::wayland::wayland_frame_meta::WaylandFrameMeta;
use crate::backend::wayland::wayland_geometry::WaylandGeometry;
use crate::backend::wayland::wayland_output_info::WaylandOutputInfo;
use crate::backend::wayland::wayland_output_mode::WaylandOutputMode;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_shm::{Format, WlShm};
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::WEnum;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

/**
 *   Globals as advertised by the Wayland compositor.
 */
const WL_SHM: &str = "wl_shm";
const WL_OUTPUT: &str = "wl_output";
const ZWLR_SCREENCOPY_MANAGER_V1: &str = "zwlr_screencopy_manager_v1";

///
/// This Struct holds State while operating with the wayland compositor.
/// It is available on any event handling and used to store the information we got from the compositor.
/// It also implements the event handlers for the event queue.
///
/// Note that many properties are options as they are first available when the corresponding event triggers.
///
#[derive(Default, Clone, Debug)]
pub struct WaylandScreenshotState {
    pub outputs_fetched: bool,
    // if the compositor did notify us about all available outputs
    pub outputs: Vec<WaylandOutputInfo>,

    // per screenshot
    pub screenshot_ready: bool,
    // true when `zwlr_screencopy_manager_v1` finished screenshotting.
    pub current_frame: Option<WaylandFrameMeta>, // holds metadata of the current screenshot

    pub wl_shm: Option<WlShm>,
    // shared memory managemant
    pub shm_formats: Vec<Format>, // supported shared memory formats by the compositor

    pub zwlr_screencopy_manager_v1: Option<ZwlrScreencopyManagerV1>,
}

impl WaylandScreenshotState {
    /**
     * Resets state to before a screenshot was made.
     */
    pub fn next_screen(&mut self) {
        self.screenshot_ready = false;
        self.current_frame = None;
    }
}

// #########################
// # Wayland Event Handler #
// #########################

/// Handles events regarding an output
impl Dispatch<WlOutput, ()> for WaylandScreenshotState {
    fn event(
        state: &mut Self,
        proxy: &WlOutput,
        event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let output_query = state
            .outputs
            .iter_mut()
            .find(|output| output.output.id() == proxy.id());

        let output = match output_query {
            Some(output) => output,
            None => {
                let output_info = WaylandOutputInfo::from_wl_output(proxy.clone());
                state.outputs.push(output_info);

                state.outputs.last_mut().unwrap()
            }
        };

        match event {
            wl_output::Event::Name { name } => output.name = name,
            wl_output::Event::Description { description } => output.description = description,
            wl_output::Event::Scale { factor } => output.scale = factor,
            wl_output::Event::Geometry { .. } => {
                output.geometry = WaylandGeometry::from_wayland_geometry(event).unwrap()
            }
            wl_output::Event::Mode { .. } => {
                output.mode = WaylandOutputMode::from_wayland_event(event).unwrap()
            }
            wl_output::Event::Done => state.outputs_fetched = true,
            _ => (),
        }
    }
}

/// Triggered when compositor notifies us about globals.
/// We bind here to the globals we need.
impl Dispatch<wl_registry::WlRegistry, ()> for WaylandScreenshotState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        // bind to global events
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                WL_OUTPUT => {
                    registry.bind::<WlOutput, _, _>(name, version, qhandle, ());
                }
                WL_SHM => {
                    state.wl_shm = Some(registry.bind::<WlShm, _, _>(name, version, qhandle, ()))
                }
                ZWLR_SCREENCOPY_MANAGER_V1 => {
                    state.zwlr_screencopy_manager_v1 = Some(
                        registry.bind::<ZwlrScreencopyManagerV1, _, _>(name, version, qhandle, ()),
                    )
                }
                _ => (),
            }
        }
    }
}

/// Triggered when compositor notifies about a supported shared memory format.
impl Dispatch<WlShm, ()> for WaylandScreenshotState {
    fn event(
        state: &mut Self,
        _proxy: &WlShm,
        event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // We got a supported format from the Wayland Compositor
        if let wl_shm::Event::Format {format: WEnum::Value(format)} = event {
            state.shm_formats.push(format);
        }
    }
}

/// Triggered when something with the screenshot we are taking happens.
impl Dispatch<ZwlrScreencopyFrameV1, ()> for WaylandScreenshotState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zwlr_screencopy_frame_v1::Event::BufferDone = event {
            // Not interested, screenshot is ready on ready event.
        }

        if let zwlr_screencopy_frame_v1::Event::Ready { .. } = event {
            state.screenshot_ready = true;
        }

        if let zwlr_screencopy_frame_v1::Event::Buffer { .. } = event {
            state.current_frame = WaylandFrameMeta::from_wayland_event(&event);
        }

        if let zwlr_screencopy_frame_v1::Event::Failed = event {
            panic!("zwlr_screencopy_frame_v1 failed to create a screenshot.");
        }
    }
}

/// Screencopy manager events
impl Dispatch<ZwlrScreencopyManagerV1, ()> for WaylandScreenshotState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrScreencopyManagerV1,
        _event: zwlr_screencopy_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // Not interested
    }
}

/// Shared Memory pool events
impl Dispatch<WlShmPool, ()> for WaylandScreenshotState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // Not interested
    }
}

/// Shared Memory Buffer events
impl Dispatch<WlBuffer, ()> for WaylandScreenshotState {
    fn event(
        _state: &mut Self,
        _proxy: &WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // Not interested
    }
}

// # Utility functions to create wrapper types

impl WaylandGeometry {
    pub fn from_wayland_geometry(event: wl_output::Event) -> Option<WaylandGeometry> {
        if let wl_output::Event::Geometry {
            x,
            y,
            physical_width,
            physical_height,
            model,
            make,
            subpixel,
            transform,
        } = event
        {
            Some(WaylandGeometry {
                x,
                y,
                physical_width,
                physical_height,
                model,
                make,
                subpixel: match subpixel {
                    WEnum::Value(subpixel) => Some(subpixel),
                    WEnum::Unknown(_) => None,
                },
                transform: match transform {
                    WEnum::Value(transform) => Some(transform),
                    WEnum::Unknown(_) => None,
                },
            })
        } else {
            None
        }
    }
}

impl WaylandOutputMode {
    pub fn from_wayland_event(event: wl_output::Event) -> Option<Self> {
        if let wl_output::Event::Mode {
            height,
            width,
            refresh,
            flags,
        } = event
        {
            Some(Self {
                height,
                width,
                refresh,
                flags: match flags {
                    WEnum::Value(flags) => Some(flags),
                    WEnum::Unknown(_) => None,
                },
            })
        } else {
            None
        }
    }
}

impl WaylandFrameMeta {
    pub fn from_wayland_event(event: &zwlr_screencopy_frame_v1::Event) -> Option<Self> {
        if let &zwlr_screencopy_frame_v1::Event::Buffer {
            format,
            width,
            height,
            stride,
        } = event
        {
            Some(Self {
                width,
                height,
                stride,
                format: match format {
                    WEnum::Value(format) => Some(format),
                    WEnum::Unknown(_) => None,
                },
            })
        } else {
            None
        }
    }
}

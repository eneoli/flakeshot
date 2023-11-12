use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1;
use wayland_protocols_wlr::screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;
use crate::backend::geometry::Geometry;
use crate::backend::output_info::OutputInfo;
use crate::backend::output_mode::OutputMode;

/**
 *   Globals as advertised by the Wayland compositor.
 */
const WL_SHM: &'static str = "wl_shm";
const WL_OUTPUT: &'static str = "wl_output";
const ZWLR_SCREENCOPY_MANAGER_V1: &'static str = "zwlr_screencopy_manager_v1";

#[derive(Default, Debug)]
pub struct WaylandScreenshotState {
    pub outputs_fetched: bool,
    pub screenshot_ready: bool,

    pub outputs: Vec<OutputInfo>,

    pub wl_shm: Option<WlShm>,
    // pub shn_formats: Vec<>, // TODO check how format looks like

    pub zwlr_screencopy_manager_v1: Option<ZwlrScreencopyManagerV1>,
}

impl Dispatch<WlOutput, ()> for WaylandScreenshotState {
    fn event(
        state: &mut Self,
        proxy: &WlOutput,
        event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let output_query = state.outputs
            .iter_mut()
            .find(|output| output.output.id() == proxy.id());

        let output = match output_query {
            Some(output) => output,
            None => {
                let output_info = OutputInfo::from_wl_output(proxy.clone());
                state.outputs.push(output_info);

                state.outputs.last_mut().unwrap()
            }
        };

        match event {
            wl_output::Event::Name { name } => output.name = name,
            wl_output::Event::Description { description } => output.description = description,
            wl_output::Event::Scale { factor } => output.scale = factor,
            wl_output::Event::Geometry { .. } => output.geometry = Geometry::from_wayland_geometry(event).unwrap(),
            wl_output::Event::Mode { .. } => output.mode = OutputMode::from_wayland_event(event).unwrap(),
            wl_output::Event::Done => state.outputs_fetched = true,
            _ => (),
        }
    }
}

// #########################
// # Wayland Event Handler #
// #########################

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
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                WL_OUTPUT => {
                    registry.bind::<WlOutput, _, _>(name, version, qhandle, ());
                },
                WL_SHM => state.wl_shm = Some(
                    registry.bind::<WlShm, _, _>(name, version, qhandle, ())
                ),
                ZWLR_SCREENCOPY_MANAGER_V1 => state.zwlr_screencopy_manager_v1 = Some(
                    registry.bind::<ZwlrScreencopyManagerV1, _, _>(name, version, qhandle, ())
                ),
                _ => (),
            }
        }
    }
}

impl Dispatch<WlShm, ()> for WaylandScreenshotState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {

        //println!("Advertised Format for SHM");
        //println!("{:#?}", event);
    }
}

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
            state.screenshot_ready = true;
        }

        if let zwlr_screencopy_frame_v1::Event::Buffer { .. } = event {
           // println!("{}", width); // TODO save infos
        }

        if let zwlr_screencopy_frame_v1::Event::Ready { .. } = event {
            println!("Buffer ready"); // TODO do we need this?
        }

        if let zwlr_screencopy_frame_v1::Event::Failed = event {
            panic!("zwlr_screencopy_frame_v1 failed to create a screenshot.");
        }
    }
}

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

// # Utility functions to create wrapper types

impl Geometry {
    pub fn from_wayland_geometry(event: wl_output::Event) -> Option<Geometry> {
        if let wl_output::Event::Geometry {
            x,
            y,
            physical_width,
            physical_height,
            model,
            make,
            ..
        } = event {
            Some(
                Geometry {
                    x,
                    y,
                    physical_width,
                    physical_height,
                    model,
                    make,
                }
            )
        } else {
            None
        }
    }
}

impl OutputMode {
    pub fn from_wayland_event(event: wl_output::Event) -> Option<Self> {
        if let wl_output::Event::Mode {
            height,
            width,
            refresh,
            ..
        } = event {
            Some(
                Self {
                    height,
                    width,
                    refresh,
                }
            )
        } else {
            None
        }
    }
}
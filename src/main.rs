use std::fmt::Debug;
use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle, Proxy};
use wayland_client::protocol::{wl_output, wl_shm};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1;
use wayland_protocols_wlr::screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;
use flakeshot::backend::geometry::Geometry;
use flakeshot::backend::output_info::OutputInfo;
use flakeshot::backend::output_mode::OutputMode;

const WL_SHM: &'static str = "wl_shm";

const WL_OUTPUT: &'static str = "wl_output";

const ZWLR_SCREENCOPY_MANAGER_V1: &'static str = "zwlr_screencopy_manager_v1";

struct AppData {
    registry: WlRegistry,
    wl_shm: Option<WlShm>,
    wl_output: Option<WlOutput>,
    zwlr_screencopy_manager_v1: Option<ZwlrScreencopyManagerV1>,

    outputs:Vec<OutputInfo>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &wl_registry::WlRegistry,
             event: wl_registry::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            println!("[{}] {} (v{})", name, interface, version);

            match interface.as_str() {
                WL_SHM => { state.wl_shm = Some(state.registry.bind::<WlShm, _, _>(name, version, qhandle, ())) }
                WL_OUTPUT => { state.wl_output = Some(state.registry.bind::<WlOutput, _, _>(name, version, qhandle, ())) }
                ZWLR_SCREENCOPY_MANAGER_V1 => { state.zwlr_screencopy_manager_v1 = Some(state.registry.bind::<ZwlrScreencopyManagerV1, _, _>(name, version, qhandle, ())) }
                default => (),
            }
        }
    }
}

impl Dispatch<WlShm, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &WlShm,
             event: wl_shm::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>) {
        println!("SHM");
    }
}

impl Dispatch<ZwlrScreencopyManagerV1, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &ZwlrScreencopyManagerV1,
             event: zwlr_screencopy_manager_v1::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>) {
        println!("Screencopy Manager");
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &ZwlrScreencopyFrameV1,
             event: zwlr_screencopy_frame_v1::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>) {
        println!("Something happened");
    }
}

impl Dispatch<WlOutput, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &WlOutput,
             event: wl_output::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>
    ) {

        let mut outputQuery = state.outputs
            .iter_mut()
            .find(|output| output.output.id() == proxy.id() );

        let output: &mut OutputInfo = match outputQuery {
            Some(output) => output,
            None => {
                let mut output_info = OutputInfo::from_wl_output(proxy.clone());
                state.outputs.push(output_info);

                state.outputs.last_mut().unwrap()
            },
        };

        match event {
            wl_output::Event::Name {name} => output.name = name,
            wl_output::Event::Description {description} => output.description = description,
            wl_output::Event::Scale {factor} => output.scale = factor,
            wl_output::Event::Geometry {..} => output.geometry = Geometry::from_wayland_geometry(event).unwrap(),
            wl_output::Event::Mode {..} => output.mode = OutputMode::from_wayland_event(event).unwrap(),
            default => (),
        }
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();
    let mut queue = conn.new_event_queue();
    let queue_handle = queue.handle();

    let registry = display.get_registry(&queue_handle, ());

    let mut appData: AppData = AppData {
        registry,
        wl_output: None,
        wl_shm: None,
        zwlr_screencopy_manager_v1: None,
        outputs: vec![],
    };

    queue.roundtrip(&mut appData).unwrap();

    //appData.zwlr_screencopy_manager_v1.unwrap().

    appData.zwlr_screencopy_manager_v1.clone().unwrap().capture_output(
        0,
        &appData.wl_output.clone().unwrap(),
        &queue_handle,
        (),
    );

    queue.blocking_dispatch(&mut appData);

    println!("{:#?}", appData.outputs);

}

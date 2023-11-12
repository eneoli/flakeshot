use std::fmt::Debug;
use std::fs::File;
use std::os::fd::{AsFd};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use memmap2::MmapMut;
use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle, Proxy};
use wayland_client::protocol::{wl_buffer, wl_output, wl_shm, wl_shm_pool};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1;
use wayland_protocols_wlr::screencopy::v1::client::{zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;
use flakeshot::backend::geometry::Geometry;
use flakeshot::backend::output_info::OutputInfo;
use flakeshot::backend::output_mode::OutputMode;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_shm::Format::{Xbgr8888};
use wayland_client::protocol::wl_shm_pool::WlShmPool;

const WL_SHM: &'static str = "wl_shm";

const WL_OUTPUT: &'static str = "wl_output";

const ZWLR_SCREENCOPY_MANAGER_V1: &'static str = "zwlr_screencopy_manager_v1";

struct AppData {
    registry: WlRegistry,
    wl_shm: Option<WlShm>,
    wl_output: Option<WlOutput>,
    zwlr_screencopy_manager_v1: Option<ZwlrScreencopyManagerV1>,

    outputs: Vec<OutputInfo>,

    ready: bool,
}

fn create_shm_file(name: &'static str, bytes: u64) -> anyhow::Result<File> {
    let options = memfd::MemfdOptions::default()
        .allow_sealing(true);

    let mfd = options.create(name)?;

    mfd.as_file().set_len(bytes)?;

    mfd.add_seals(&[
        memfd::FileSeal::SealShrink,
        memfd::FileSeal::SealGrow,
        memfd::FileSeal::SealSeal,
    ])?;

    Ok(mfd.into_file())
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

            //println!("{:#?}",interface);


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


        //println!("Advertised Format for SHM");
        //println!("{:#?}", event);
    }
}

impl Dispatch<WlShmPool, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &WlShmPool,
             event: wl_shm_pool::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>,
    ) {}
}

impl Dispatch<WlBuffer, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &WlBuffer,
             event: wl_buffer::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>,
    ) {}
}

impl Dispatch<ZwlrScreencopyManagerV1, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &ZwlrScreencopyManagerV1,
             event: zwlr_screencopy_manager_v1::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>) {}
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &ZwlrScreencopyFrameV1,
             event: zwlr_screencopy_frame_v1::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>) {
        println!("Buffer!!!");
        if let zwlr_screencopy_frame_v1::Event::BufferDone = event {
            state.ready = true;
        }

        if let zwlr_screencopy_frame_v1::Event::Failed = event {
            println!("FAILED!");
        }

        if let zwlr_screencopy_frame_v1::Event::Buffer { width, .. } = event {
            println!("{}", width);
        }

        if let zwlr_screencopy_frame_v1::Event::Ready { .. } = event {
            println!("Buffer ready");
        }
    }
}

impl Dispatch<WlOutput, ()> for AppData {
    fn event(state: &mut Self,
             proxy: &WlOutput,
             event: wl_output::Event,
             data: &(),
             conn: &Connection,
             qhandle: &QueueHandle<Self>,
    ) {
        let mut output_query = state.outputs
            .iter_mut()
            .find(|output| output.output.id() == proxy.id());

        let output = match output_query {
            Some(output) => output,
            None => {
                let mut output_info = OutputInfo::from_wl_output(proxy.clone());
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
        ready: false,
    };

    queue.roundtrip(&mut appData).unwrap();

    // setup shm
    let mut memfile = create_shm_file("/tmp/flakeshot0000", 2160 * 3840 * 4).unwrap();
    let shm_pool = appData.wl_shm.as_ref().unwrap().create_pool(memfile.as_fd(), 3840 * 2160 * 4, &queue_handle, ());

    // TODO ka ob format richtig
    let buffer: WlBuffer = shm_pool.create_buffer(0, 3840, 2160, 3840 * 4, Xbgr8888, &queue_handle, ());

    loop {
        queue.blocking_dispatch(&mut appData).unwrap();

        if appData.outputs.len() > 0 {

            //println!("SCREENSHOT!");

            println!("{:#?}", appData.outputs.last_mut().unwrap().description);

            let frame = appData.zwlr_screencopy_manager_v1
                .as_ref()
                .unwrap()
                .capture_output(
                    0,
                    &appData.outputs.last_mut().unwrap().output,
                    &queue_handle,
                    (),
                );

            frame.copy(&buffer);

            while (!appData.ready) {
                queue.blocking_dispatch(&mut appData).unwrap();
            }

            sleep(Duration::from_secs(5));

            let mut frame_mmap = unsafe { MmapMut::map_mut(&memfile).unwrap() };
            let data = &mut *frame_mmap;

            // println!("{:#?}", data.to_vec());

            image::save_buffer(&Path::new("/home/oliver/screen.png"), data, 3840, 2160, image::ColorType::Rgba8).expect("TODO: panic message");

            //println!("{:#?}", frame);

            break;
        }
    }

    //println!("{:#?}", appData.outputs);
}

use std::ops::RangeInclusive;

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_registry::{self, WlRegistry},
        wl_seat,
    },
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1;

#[derive(Debug)]
struct AppData(pub String);

impl Dispatch<WlRegistry, GlobalListContents> for AppData {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        data: &GlobalListContents,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        println!("something");
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        obj: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        data: &(),
        conn: &Connection,
        handle: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global { interface, .. } = event {}
    }
}
pub fn get_wl_backend() {
    let backend = String::from("None");
    let mut data = AppData(backend);
    // connection to wayland server
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut queue) = registry_queue_init::<AppData>(&conn).unwrap();
    let handle = queue.handle();
    //let mut seats = Vec::new();
    globals.contents().with_list(|list| {
        for global in list {
            dbg!(global);
        }
    });
    let manager =
        globals.bind::<ZwlrDataControlManagerV1, _, _>(&handle, RangeInclusive::new(0, 1), ());

    if manager.is_err() {
        return;
    }
    let manager = manager.unwrap();

    let mut gg = String::from("pingpang");
    manager.get_data_device(seat, &handle, gg);
    //let u_data =
    manager.create_data_source(&handle, gg)
}

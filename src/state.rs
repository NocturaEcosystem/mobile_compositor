use std::ffi::OsString;

use calloop::LoopSignal;
use smithay::{desktop::{PopupManager, Space, Window}, input::{Seat, SeatState}, reexports::wayland_server::DisplayHandle, wayland::{compositor::{CompositorClientState, CompositorState}, output::OutputManagerState, selection::data_device::DataDeviceState, shell::xdg::XdgShellState, shm::ShmState}};

mod compositor_impls;
pub struct NocturaStates {
    pub time: std::time::Instant,
    pub dh: DisplayHandle,
    pub ls: LoopSignal,
    pub socket_name: OsString,
    pub ss: SeatState<Self>,
    pub seat: Seat<Self>,
    pub output_manager: OutputManagerState,
    pub popups: PopupManager,
    pub comp_state: CompositorState,
    pub shm_state: ShmState,
    pub dds: DataDeviceState,
    pub xdg_state: XdgShellState,
    pub space: Space<Window>,
}

mod client_impls;
#[derive(Default)]
pub struct NocturaClients {  // this will contain data about our clients/apps
    pub comp_state: CompositorClientState,
}

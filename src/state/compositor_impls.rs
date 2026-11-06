use calloop::EventLoop;
use smithay::{delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell, input::SeatHandler, reexports::wayland_server::{Client, Display, protocol::{wl_buffer, wl_surface::WlSurface}}, wayland::{buffer::BufferHandler, compositor::{CompositorClientState, CompositorHandler}, output::OutputHandler, selection::{SelectionHandler, data_device::{self, ClientDndGrabHandler, DataDeviceHandler, ServerDndGrabHandler}}, shell::xdg::XdgShellHandler, shm::ShmHandler, socket::ListeningSocketSource}};
use tracing::info;

use crate::state::{NocturaStates, NocturaClients};
use smithay::{desktop::{PopupManager, Space, Window}, input::{Seat, SeatState}, reexports::wayland_server::DisplayHandle, wayland::{compositor::CompositorState, output::OutputManagerState, selection::data_device::DataDeviceState, shell::xdg::XdgShellState, shm::ShmState}};
use smithay::reexports::wayland_server::Resource;

// Traits:

/* 
    In smithay we have traits, which can be given to structs. for example,
    because I am using the seat field in my sturct, i have to give it the seathandler trait
*/

impl SeatHandler for NocturaStates {
    type PointerFocus = WlSurface;
    type KeyboardFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.ss
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: smithay::input::pointer::CursorImageStatus) {
      //  self.cs = image
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let client = focused.and_then(|surface| {
            self.dh.get_client(surface.id()).ok()   // basically, if there was a surface, we need the client
                                                    // this code gets the client who owns the surface
        });
        data_device::set_data_device_focus(&self.dh, seat, client);
    }
}

impl DataDeviceHandler for NocturaStates {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.dds
    }
}
impl ClientDndGrabHandler for NocturaStates {}
impl ServerDndGrabHandler for NocturaStates {}
impl SelectionHandler for NocturaStates {
    type SelectionUserData = ();
}

impl OutputHandler for NocturaStates {}

impl CompositorHandler for NocturaStates {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.comp_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<NocturaClients>().unwrap().comp_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        // TODO
    }
}

impl ShmHandler for NocturaStates {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl BufferHandler for NocturaStates {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl XdgShellHandler for NocturaStates {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_state
    }
    
    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // TODO
    }
    
    fn new_popup(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, positioner: smithay::wayland::shell::xdg::PositionerState) {
        // TODO
    }

    fn grab(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat, serial: smithay::utils::Serial) {
        // TODO
    }

    fn reposition_request(&mut self, surface: smithay::wayland::shell::xdg::PopupSurface, positioner: smithay::wayland::shell::xdg::PositionerState, token: u32) {
        // TODO
    }
}

// Delegate

/* 
    In smitahy, we 'delegate' traits.
    delegate means to tell smithay "I applied the traits and use my functions for your protocol"
    so for example, if smithay needs to check your seat state to do an operation, it will check the
    trait 'SeatHandler', function 'seat_state' where you return the Seat State
*/

delegate_output!(NocturaStates);
delegate_seat!(NocturaStates);
delegate_compositor!(NocturaStates);
delegate_shm!(NocturaStates);
delegate_data_device!(NocturaStates);
delegate_xdg_shell!(NocturaStates);
// My own implementations:

/*
    Along with traits i added my own functions to my struct. I did this to make my code easier
    to debug
    
    For people who are used to clasical OOP,
    class mycalss {
        MYVAR = 10

        def myMethod(&self) {
            return self.MYVAR
        }
    }

    in rust is

    struct myclass {
        MYVAR: i32 // you have to define the type, NOT GIVE VALUE
    }

    impl myclass {            // add methods to my class, 'myclass'
        fn new() -> Self  {   // for any new instance of class that wants to run like so: myclass::new(),
//                               but technically, you could also just do myclass { MYVAR: 10 }
            return Self{
                10
            }
        }

        fn myMethod(&self) -> i32 {
            reuturn self.MYVAR
        }
    }
*/

impl NocturaStates {
    pub fn new(display: &Display<Self>, el: &EventLoop<Self>) -> Self {
        let dh: smithay::reexports::wayland_server::DisplayHandle = display.handle(); // create a 'key' to let your compositor talk to wayland server
        let mut ss = SeatState::new();
        let mut seat: Seat<Self> = ss.new_wl_seat(&dh, "Noctura_seat");
        let output_manager: OutputManagerState = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let comp_state = CompositorState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let dds = DataDeviceState::new::<Self>(&dh);
        let listen_source = ListeningSocketSource::new_auto().unwrap();
        let socket_name = listen_source.socket_name().to_os_string();
        info!("SOCKET NAME: {:?}", socket_name);
        let xdg_state = XdgShellState::new::<Self>(&dh);
        let space = Space::default();
        let popups = PopupManager::default();

        Self::prep_seat(&mut seat);

        Self {
            dh,
            ls: el.get_signal(),
            socket_name,
            ss,
            seat,
            output_manager,
            popups,
            comp_state,
            shm_state,
            dds,
            xdg_state,
            space
        }
    }
    
    fn prep_seat(seat: &mut Seat<Self>) {
        seat.add_touch();
    }
}
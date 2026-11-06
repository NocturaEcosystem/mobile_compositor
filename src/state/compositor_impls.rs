use std::time::Duration;

use calloop::EventLoop;
use smithay::{backend::{renderer::{self, damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement, gles::GlesRenderer}, winit::{self, WinitEvent}}, delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell, desktop::space, input::SeatHandler, output::{Mode, Output, PhysicalProperties}, reexports::wayland_server::{Client, Display, protocol::{wl_buffer, wl_surface::WlSurface}}, utils::{Scale, Transform::Flipped180}, wayland::{buffer::BufferHandler, compositor::{CompositorClientState, CompositorHandler}, output::OutputHandler, selection::{SelectionHandler, data_device::{self, ClientDndGrabHandler, DataDeviceHandler, ServerDndGrabHandler}}, shell::xdg::XdgShellHandler, shm::ShmHandler, socket::ListeningSocketSource}};
use tracing::info;

use crate::state::{NocturaStates, NocturaClients};
use smithay::{desktop::{PopupManager, Space, Window}, input::{Seat, SeatState}, reexports::wayland_server::DisplayHandle, wayland::{compositor::CompositorState, output::OutputManagerState, selection::data_device::DataDeviceState, shell::xdg::XdgShellState, shm::ShmState}};
use smithay::reexports::wayland_server::Resource;
use tracing::warn;

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
        let time = std::time::Instant::now();

        Self::prep_seat(&mut seat);

        Self {
            time,
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

    pub fn start_win(&mut self, el: &mut EventLoop<NocturaStates>) -> Result<(), Box<dyn std::error::Error>> {
        // this function is used to open the window through which you can see Noctura compositor

        let (mut backend, winit) = winit::init::<GlesRenderer>()?;
        let mut out_mode = Mode {
            size: backend.window_size(),
            refresh: 60000,
        };
        let output = Output::new(
            "noctura".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: smithay::output::Subpixel::Unknown,
                // NOTE: add real monitor name + model in the non-nested version
                make: "NocturaDisplay".into(),
                model: "Nested".into(),
            }
        );

        let _ = output.create_global::<NocturaStates>(&self.dh);
        output.change_current_state(
            Some(out_mode),
            Some(Flipped180),      // NOTE: I currently dont know why i have to flip 
    //                                                  the display, but for openGL you must do it, im not sure about other renderers
            None,
            Some((0, 0).into())
        );
        output.set_preferred(out_mode);
        self.space.map_output(&output, (0, 0));
        let mut damage_tracker = OutputDamageTracker::from_output(&output);
        el.handle().insert_source(winit, move |event, _, state| {
            match event {
                WinitEvent::CloseRequested => {
                    state.ls.stop();
                }
                WinitEvent::Resized { size, scale_factor } => {
                    output.change_current_state(Some(Mode {size, refresh: 60_000}),
                        None,
                        None,
                        None,
                    );
                }
                WinitEvent::Redraw => {
                    let render_res = backend.bind().and_then(|(renderer, mut framebuffer)| {
                        let scale = Scale::from(output.current_scale().fractional_scale());
                        space::render_output::<
                            _,
                            WaylandSurfaceRenderElement<GlesRenderer>,
                            _,
                            _
                        >(
                            &output, renderer, &mut framebuffer,
                            1.0, 0, [&state.space],
                            &[],
                            &mut damage_tracker,
                            [0.7, 0.1, 1.0, 1.0]
                        ).map_err(|err| match err {
                            renderer::damage::Error::Rendering(err) => smithay::backend::SwapBuffersError::from(err),
                            _ => unreachable!(),
                        })
                    });

                    match render_res {
                        Ok(ror) => {
                            if let Some(damage) = ror.damage {
                                backend.submit(Some(damage));
                            }
                        }
                        Err(error) => {
                            warn!("Error while rendering {}", error)
                        }
                    }
    

                    for win in state.space.elements() {
                        win.send_frame(&output, state.time.elapsed(), Some(Duration::ZERO), |_, _| { 
                            Some(output.clone())   // clone as &output already borrowed it
                        });
                    }

                    state.space.refresh();
                    state.dh.flush_clients();
                    backend.window().request_redraw();
                }
                WinitEvent::Input(e) => {
                   // state.handle_input(e);
                }
                _ => {}
            }
        }).unwrap();
        Ok(())
    }
    
}
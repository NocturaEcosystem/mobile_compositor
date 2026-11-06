use calloop::{EventLoop, Interest, generic::Generic};
use smithay::reexports::{wayland_server::Display};
mod state;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().compact().init();
    let mut event_loop = EventLoop::<state::NocturaStates>::try_new()?;
    let display: Display<state::NocturaStates> = Display::new()?;
    let noctura_comp =  state::NocturaStates::new(&display, &event_loop);
    event_loop.handle().clone()
        .insert_source(
            Generic::new(display, Interest::READ, calloop::Mode::Level),
            |_, display, state| {
                // Safety: we don't drop the display
                unsafe {
                    display.get_mut().dispatch_clients(state).unwrap();
                }
                Ok(calloop::PostAction::Continue)
            },
        )
        .expect("Error while trying to initializing wayland server source");
    Ok(())
}

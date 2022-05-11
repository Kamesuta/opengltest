mod support;

extern crate vlc;

use std::sync::mpsc::channel;
use vlc::Event as VlcEvent;
use vlc::{EventType, Instance, Media, MediaPlayer, State};

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("A fantastic window!");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    struct GameState {
        pos: [f64; 2],
    }

    let gl = support::load(&windowed_context.context());
    let mut state = GameState { pos: [0.0, 0.0] };

    el.run(move |event, _, control_flow| {
        //println!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => state.pos = [position.x, position.y],
                _ => (),
            },
            Event::RedrawRequested(_) => {
            }
            _ => (),
        }
        
        gl.draw_frame([1.0, 0.5, 0.7, 1.0], state.pos);
        windowed_context.swap_buffers().unwrap();
    });

    let args: Vec<String> = std::env::args().collect();

    let path = match args.get(1) {
        Some(s) => s,
        None => {
            println!("Usage: cli_audio_player path_to_a_media_file");
            return;
        }
    };
    let instance = Instance::new().unwrap();

    let md = Media::new_path(&instance, path).unwrap();
    let mdp = MediaPlayer::new(&instance).unwrap();
    let (tx, rx) = channel::<()>();
    let em = md.event_manager();
    let _ = em.attach(EventType::MediaStateChanged, move |e, _| match e {
        VlcEvent::MediaStateChanged(s) => {
            println!("State : {:?}", s);
            if s == State::Ended || s == State::Error {
                tx.send(()).unwrap();
            }
        }
        _ => (),
    });
    mdp.set_media(&md);
    // Start playing
    mdp.play().unwrap();
    // Wait for end state
    rx.recv().unwrap();
}

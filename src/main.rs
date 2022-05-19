mod media;
mod support;

extern crate vlc;

use std::os::windows::prelude::OsStringExt;
use std::sync::mpsc::channel;
use std::time::Instant;
use vlc::Event as VlcEvent;
use vlc::{EventType, Instance, Media, MediaPlayer, State};

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use libc::c_void;
use media::{MediaExt, MediaPlayerExt};
use std::sync::{Arc, Mutex};

use alto::{Alto, Mono, Source};
use std::ffi::OsString;

const TARGET_FPS: u64 = 60;

fn main() -> Result<(), String> {
    let alto = Alto::load_default().unwrap();

    // for s in alto.enumerate_outputs() {
    //     println!("Found device: {}", w.to_str().map_err(|e| e.to_string())?);
    // }
    
    let device = alto.open(None).unwrap(); // Opens the default audio device
    let context = device.new_context(None).unwrap(); // Creates a default context
    
    // Configure listener
    context.set_position([1.0, 4.0, 5.0]).unwrap();
    context.set_velocity([2.5, 0.0, 0.0]).unwrap();
    context.set_orientation(([0.0, 0.0, 1.0], [0.0, 1.0, 0.0])).unwrap();
    
    // Now you can load your samples and store them in a buffer with
    // `context.new_buffer(samples, frequency)`;

    let freq = 44100;
    let key_freq = 440.0;
    let mut samples: Vec<i16> = vec![0; freq];
    for i in 0..samples.len() {
        samples[i] = ((key_freq * std::f32::consts::PI * 2.0 * i as f32 / freq as f32).sin() * i16::MAX as f32) as i16;
    }
    let buf = context.new_buffer::<Mono<i16>, _>(samples, freq as i32).unwrap();
    let buf = Arc::new(buf);
    let mut source = context.new_static_source().unwrap();
    source.set_buffer(buf).unwrap();
    source.play();

    // TODO: Linux, Mac対応
    // TODO: Audio OpenAL
    // OK: YouTube対応
    // OK: 一時停止したときにポーズされるようにする
    let args: Vec<String> = std::env::args().collect();

    let path = match args.get(1) {
        Some(s) => s,
        None => {
            return Err("No media file specified".to_string());
        }
    };
    let instance = Instance::new().ok_or("Failed to create instance")?;

    let md = Media::new_location(&instance, path).ok_or("Failed to create media")?;
    let mdp = MediaPlayer::new(&instance).ok_or("Failed to create media player")?;

    struct VlcContext {
        pixel_buffer: Vec<u32>,
        need_update: bool,
        locked: bool,
    }
    
    let (video_width, video_height) = (512, 512);
    mdp.set_video_format(
        "RV24",
        video_width,
        video_height,
        video_width * 3,
    );
    let context = Arc::new(Mutex::new(VlcContext {
        pixel_buffer: vec![0; video_width as usize * video_height as usize],
        need_update: false,
        locked: false,
    }));
    let c1 = Arc::clone(&context);
    let c2 = Arc::clone(&context);
    mdp.set_video_callbacks(
        move || {
            let mut context = c1.lock().unwrap();
            context.locked = true;
            context.pixel_buffer.as_mut_ptr() as *mut c_void
        },
        Some(Box::new(move || {
            let mut context = c2.lock().unwrap();
            context.locked = false;
        })),
        Some(Box::new(|| {})),
    );

    mdp.set_audio_format("f32l", 48000, 2);
    mdp.set_callbacks(
        |_samples, count, pts| {
            println!("{} {}", count, pts);
        },
        None,
        None,
        None,
        None,
    );

    let (tx, rx) = channel::<()>();
    let em = md.event_manager();
    let _ = em.attach(EventType::MediaParsedChanged, move |e, _| match e {
        VlcEvent::MediaParsedChanged(s) => {
            match s as u32 {
                media::MediaParsedStatusDone => {
                    // Media parsed
                    tx.send(()).unwrap();
                }
                _ => {
                    println!("Media not parsed");
                }
            }
        }
        _ => (),
    });

    let md = {
        md.parse_with_options(media::MediaParseNetwork, -1)?;
        rx.recv().unwrap();
        if let Some(submd) = md.subitems().item_at_index(0) {
            submd
        } else {
            md
        }
    };

    let em = md.event_manager();
    let _ = em.attach(EventType::MediaStateChanged, move |e, _| match e {
        VlcEvent::MediaStateChanged(s) => {
            println!("State : {:?}", s);
            if s == State::Ended || s == State::Error {
                // Ended
            }
        }
        _ => (),
    });

    mdp.set_media(&md);
    // Start playing
    mdp.play().map_err(|_| "Failed to play")?;

    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("A fantastic window!");

    let windowed_context = ContextBuilder::new()
        .build_windowed(wb, &el)
        .map_err(|err| err.to_string())?;
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    struct GameState {
        pos: [f64; 2],
    }

    let gl = support::load(&windowed_context.context());
    //gl.upload_texture_img("res/tuku.png");
    let mut state = GameState { pos: [0.0, 0.0] };

    el.run(move |event, _, control_flow| {
        //println!("{:?}", event);
        let start_time = Instant::now();

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => state.pos = [position.x, position.y],
                _ => (),
            },
            Event::RedrawRequested(_) => {
                match context.try_lock() {
                    Ok(mut mutex) => {
                        let mut context = &mut *mutex;
                        if !context.locked {
                            unsafe {
                                gl.upload_texture(
                                    context.pixel_buffer.as_ptr() as *const _,
                                    video_width,
                                    video_height,
                                );
                            }
                            context.need_update = false;
                        }
                    }
                    Err(_) => (),
                };
                gl.draw_frame([1.0, 0.5, 0.7, 1.0], state.pos);
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }

        match *control_flow {
            ControlFlow::Exit => (),
            _ => {
                /*
                 * Grab window handle from the display (untested - based on API)
                 */
                windowed_context.window().request_redraw();
                /*
                 * Below logic to attempt hitting TARGET_FPS.
                 * Basically, sleep for the rest of our milliseconds
                 */
                let elapsed_time = Instant::now().duration_since(start_time).as_millis() as u64;

                let wait_millis = match 1000 / TARGET_FPS >= elapsed_time {
                    true => 1000 / TARGET_FPS - elapsed_time,
                    false => 0,
                };
                let new_inst = start_time + std::time::Duration::from_millis(wait_millis);
                *control_flow = ControlFlow::WaitUntil(new_inst);
            }
        }
    });
}

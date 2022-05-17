mod support;

extern crate vlc;

use std::sync::mpsc::channel;
use std::time::Instant;
use vlc::Event as VlcEvent;
use vlc::{EventType, Instance, Media, MediaPlayer, State};

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use libc::c_void;
use std::ffi::CString;
use std::sync::Mutex;
use vlc_sys as sys;

const TARGET_FPS: u64 = 60;

struct VlcCallback {
    pixel_buffer: Vec<u32>,
    need_update: bool,
    video_width: u32,
    video_height: u32,
    vlc_mutex: Mutex<bool>,
}

impl VlcCallback {
    fn new(video_width: u32, video_height: u32) -> Self {
        VlcCallback {
            pixel_buffer: vec![0; video_width as usize * video_height as usize],
            need_update: false,
            video_width,
            video_height,
            vlc_mutex: Mutex::new(false),
        }
    }

    fn register(&mut self, player: &MediaPlayer) {
        unsafe {
            let c_str = CString::new("RV24").unwrap();
            sys::libvlc_video_set_format(
                player.raw(),
                c_str.as_ptr(),
                self.video_width,
                self.video_height,
                self.video_width * 3,
            );
            sys::libvlc_video_set_callbacks(
                player.raw(),
                Some(Self::vlc_lock),
                Some(Self::vlc_unlock),
                Some(Self::vlc_display),
                self as *mut _ as *mut c_void,
            );
        }
    }

    unsafe extern "C" fn vlc_lock(opaque: *mut c_void, planes: *mut *mut c_void) -> *mut c_void {
        let this: &mut VlcCallback = &mut *(opaque as *mut VlcCallback);
        *(this.vlc_mutex.lock().unwrap()) = true;
        *planes = this.pixel_buffer.as_mut_ptr() as *mut c_void;
        return std::ptr::null_mut();
    }
    unsafe extern "C" fn vlc_unlock(
        opaque: *mut c_void,
        picture: *mut c_void,
        planes: *const *mut c_void,
    ) {
        let this: &mut VlcCallback = &mut *(opaque as *mut VlcCallback);
        this.need_update = true;
        *(this.vlc_mutex.lock().unwrap()) = false;
    }

    extern "C" fn vlc_display(opaque: *mut c_void, picture: *mut c_void) {}
}

fn main() {
    // TODO: Linux, Mac対応
    // OK: YouTube対応
    // OK: 一時停止したときにポーズされるようにする
    let args: Vec<String> = std::env::args().collect();

    let path = match args.get(1) {
        Some(s) => s,
        None => {
            println!("Usage: cli_audio_player path_to_a_media_file");
            return;
        }
    };
    let instance = Instance::new().unwrap();

    let md = Media::new_location(&instance, path).unwrap();
    let mdp = MediaPlayer::new(&instance).unwrap();

    let mut callback = VlcCallback::new(512, 512);
    callback.register(&mdp);

    let (tx, rx) = channel::<()>();
    let em = md.event_manager();
    let _ = em.attach(EventType::MediaParsedChanged, move |e, _| match e {
        VlcEvent::MediaParsedChanged(s) => {
            match s as u32 {
                sys::libvlc_media_parsed_status_t_libvlc_media_parsed_status_done => {
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

    let md = unsafe {
        sys::libvlc_media_parse_with_options(
            md.raw(),
            sys::libvlc_media_parse_flag_t_libvlc_media_parse_network,
            -1,
        );
        rx.recv().unwrap();
        let subitems: *mut sys::libvlc_media_list_t = sys::libvlc_media_subitems(md.raw());
        let media = sys::libvlc_media_list_item_at_index(subitems, 0);
        struct MediaStruct {
            ptr: *mut sys::libvlc_media_t,
        }
        let media_st = MediaStruct { ptr: media };
        assert!(media_st.ptr != std::ptr::null_mut());
        let md: Media = std::mem::transmute(media_st);
        md
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
    mdp.play().unwrap();

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
                match callback.vlc_mutex.try_lock() {
                    Ok(locked) => {
                        if !*locked {
                            unsafe {
                                gl.upload_texture(
                                    callback.pixel_buffer.as_ptr() as *const _,
                                    callback.video_width,
                                    callback.video_height,
                                );
                            }
                            callback.need_update = false;
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

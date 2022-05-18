use libc::c_void;
use std::mem::transmute;
use std::ffi::CString;
use vlc::{Media, MediaPlayer};
use vlc_sys as sys;

pub use sys::libvlc_media_parse_flag_t_libvlc_media_parse_network as MediaParseNetwork;
pub use sys::libvlc_media_parsed_status_t_libvlc_media_parsed_status_done as MediaParsedStatusDone;

pub trait MediaExt {
    fn from_raw(media: *mut sys::libvlc_media_t) -> Self;

    fn parse_with_options(
        &self,
        parse_flag: sys::libvlc_media_parse_flag_t,
        timeout: i32,
    ) -> Result<(), String>;

    fn subitems(&self) -> MediaList;
}

impl MediaExt for Media {
    fn from_raw(media: *mut sys::libvlc_media_t) -> Self {
        assert!(media != std::ptr::null_mut());
        struct MediaStruct {
            _ptr: *mut sys::libvlc_media_t,
        }
        let media_st = MediaStruct { _ptr: media };
        let md: Media = unsafe { std::mem::transmute(media_st) };
        md
    }

    fn parse_with_options(
        &self,
        parse_flag: sys::libvlc_media_parse_flag_t,
        timeout: i32,
    ) -> Result<(), String> {
        let err = unsafe { sys::libvlc_media_parse_with_options(self.raw(), parse_flag, timeout) };
        if err == 0 {
            Ok(())
        } else {
            Err("Error while parsing media".to_string())
        }
    }

    fn subitems(&self) -> MediaList {
        unsafe {
            let raw_list = sys::libvlc_media_subitems(self.raw());
            MediaList { ptr: raw_list }
        }
    }
}

pub trait MediaPlayerExt {
    fn set_video_callbacks<F>(
        &self,
        lock: F,
        unlock: Option<Box<dyn Fn() + Send + 'static>>,
        display: Option<Box<dyn Fn() + Send + 'static>>,
    ) where
        F: Fn() -> *mut c_void + Send + 'static;

    fn set_video_format(
        &self,
        chroma: &str,
        width: u32,
        height: u32,
        pitch: u32,
    );

    fn set_audio_format(
        &self,
        format: &str,
        rate: u32,
        channels: u32,
    );
}

impl MediaPlayerExt for MediaPlayer {
    fn set_video_callbacks<F>(
        &self,
        lock: F,
        unlock: Option<Box<dyn Fn() + Send + 'static>>,
        display: Option<Box<dyn Fn() + Send + 'static>>,
    ) where
        F: Fn() -> *mut c_void + Send + 'static,
    {
        let flag_unlock = unlock.is_some();
        let flag_display = display.is_some();

        let data = VideoCallbacksData {
            lock: Box::new(lock),
            unlock,
            display,
        };
        let data = Box::into_raw(Box::new(data));

        unsafe {
            sys::libvlc_video_set_callbacks(
                self.raw(),
                Some(video_cb_lock),
                if flag_unlock {
                    Some(video_cb_unlock)
                } else {
                    None
                },
                if flag_display {
                    Some(video_cb_display)
                } else {
                    None
                },
                data as *mut c_void,
            );
        }
    }

    fn set_video_format(
        &self,
        chroma: &str,
        width: u32,
        height: u32,
        pitch: u32,
    ) {
        let c_chroma = CString::new(chroma).unwrap();
        unsafe {
            sys::libvlc_video_set_format(self.raw(), c_chroma.as_ptr(), width, height, pitch);
        }
    }

    fn set_audio_format(
        &self,
        format: &str,
        rate: u32,
        channels: u32,
    ) {
        let c_format = CString::new(format).unwrap();
        unsafe {
            sys::libvlc_audio_set_format(self.raw(), c_format.as_ptr(), rate, channels);
        }
    }
}

pub struct MediaList {
    ptr: *mut sys::libvlc_media_list_t,
}

impl MediaList {
    pub fn item_at_index(&self, i_pos: i32) -> Option<Media> {
        let ptr = unsafe { sys::libvlc_media_list_item_at_index(self.ptr, i_pos) };
        if ptr != std::ptr::null_mut() {
            Some(Media::from_raw(ptr))
        } else {
            None
        }
    }
}

impl Drop for MediaList {
    fn drop(&mut self) {
        unsafe { sys::libvlc_media_list_release(self.ptr) };
    }
}

// For video_set_callbacks
struct VideoCallbacksData {
    lock: Box<dyn Fn() -> *mut c_void + Send + 'static>,
    unlock: Option<Box<dyn Fn() + Send + 'static>>,
    display: Option<Box<dyn Fn() + Send + 'static>>,
}

unsafe extern "C" fn video_cb_lock(data: *mut c_void, planes: *mut *mut c_void) -> *mut c_void {
    let data: &VideoCallbacksData = transmute(data as *mut VideoCallbacksData);
    *planes = (data.lock)();
    return std::ptr::null_mut();
}

unsafe extern "C" fn video_cb_unlock(
    data: *mut c_void,
    _picture: *mut c_void,
    _planes: *const *mut c_void,
) {
    let data: &VideoCallbacksData = transmute(data as *mut VideoCallbacksData);
    (data.unlock.as_ref().unwrap())();
}

unsafe extern "C" fn video_cb_display(data: *mut c_void, _picture: *mut c_void) {
    let data: &VideoCallbacksData = transmute(data as *mut VideoCallbacksData);
    (data.display.as_ref().unwrap())();
}

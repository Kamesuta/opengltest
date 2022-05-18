use vlc::Media;
use vlc_sys as sys;

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

pub struct MediaList {
    ptr: *mut sys::libvlc_media_list_t,
}

impl MediaList {
    pub fn item_at_index(&self, i_pos: i32) -> Media {
        let ptr = unsafe { sys::libvlc_media_list_item_at_index(self.ptr, i_pos) };
        MediaExt::from_raw(ptr)
    }
}

impl Drop for MediaList {
    fn drop(&mut self) {
        unsafe { sys::libvlc_media_list_release(self.ptr) };
    }
}

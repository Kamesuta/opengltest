extern crate image;

use std::{path, ffi::c_void};

#[derive(Debug)]
pub struct Texture {
    id: u32,
    location: gl::types::GLenum,
    path: path::PathBuf,
}

enum TextureError {
    FileNotFound(path::PathBuf),
    InvalidFileType(String)
}

impl Texture {
    pub fn new(gl: &gl::Gl, file_path: &str, location: u32) -> Texture {
        let mut id = 0;

        let img = image::open(file_path).unwrap();
        let img = match img {
            image::DynamicImage::ImageRgb8(img) => img,
            x => x.to_rgb8()
        };
        let width = img.width();
        let height = img.height();

        unsafe {
            gl.ActiveTexture(gl::TEXTURE0 + location);
            gl.GenTextures(1, &mut id);
            gl.BindTexture(gl::TEXTURE_2D, id);

            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl.TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width as i32, height as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, (&img as &[u8]).as_ptr() as *const c_void);
            gl.GenerateMipmap(gl::TEXTURE_2D);
        }
        Texture {
            id,
            location,
            path: path::PathBuf::from(file_path)
        }
    }
}
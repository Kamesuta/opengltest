use glutin::{self, PossiblyCurrent};

use std::ffi::CStr;

pub struct Gl {
    pub gl: gl::Gl,
}

pub fn load(gl_context: &glutin::Context<PossiblyCurrent>) -> Gl {
    let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_bytes().to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL version {}", version);

    unsafe {
        let vs = gl.CreateShader(gl::VERTEX_SHADER);
        gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), std::ptr::null());
        gl.CompileShader(vs);

        let fs = gl.CreateShader(gl::FRAGMENT_SHADER);
        gl.ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), std::ptr::null());
        gl.CompileShader(fs);

        let program = gl.CreateProgram();
        gl.AttachShader(program, vs);
        gl.AttachShader(program, fs);
        gl.LinkProgram(program);
        gl.UseProgram(program);

        // VBOを生成する関数
        let mut vb = std::mem::zeroed();
        gl.GenBuffers(1, &mut vb);
        gl.BindBuffer(gl::ARRAY_BUFFER, vb);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            VERTEX_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // IBOを生成する関数
        let mut ib = std::mem::zeroed();
        gl.GenBuffers(1, &mut ib);
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib);
        gl.BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDEX_DATA.len() * std::mem::size_of::<u8>()) as gl::types::GLsizeiptr,
            INDEX_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
        let color_attrib = gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
        gl.VertexAttribPointer(
            pos_attrib as gl::types::GLuint,
            2,
            gl::FLOAT,
            0,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            std::ptr::null(),
        );
        gl.VertexAttribPointer(
            color_attrib as gl::types::GLuint,
            3,
            gl::FLOAT,
            0,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            (2 * std::mem::size_of::<f32>()) as *const () as *const _,
        );
        gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
        gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);
    }

    Gl { gl }
}

impl Gl {
    pub fn draw_frame(&self, color: [f32; 4]) {
        unsafe {
            self.gl.ClearColor(color[0], color[1], color[2], color[3]);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawElements(gl::TRIANGLES, INDEX_DATA.len() as i32, gl::UNSIGNED_BYTE, std::ptr::null());
        }
    }
}

#[rustfmt::skip]
static INDEX_DATA: [u8; 6] = [
    0, 1, 2, 0, 3, 2,
];

#[rustfmt::skip]
static VERTEX_DATA: [f32; 20] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
    -0.5,  0.5,  0.0,  1.0,  0.0,
     0.5,  0.5,  0.0,  0.0,  1.0,
     0.5, -0.5,  0.0,  0.0,  1.0,
];

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
attribute vec2 position;
attribute vec3 color;
varying vec3 v_color;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
varying vec3 v_color;
void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
\0";

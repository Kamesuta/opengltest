use glutin::{self, PossiblyCurrent};

use super::texture::Texture;
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

        let pos_attrib = gl.GetAttribLocation(program, b"pos\0".as_ptr() as *const _);
        let uv_attrib = gl.GetAttribLocation(program, b"tex_coord\0".as_ptr() as *const _);
        gl.VertexAttribPointer(
            pos_attrib as gl::types::GLuint,
            3,
            gl::FLOAT,
            0,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            std::ptr::null(),
        );
        gl.VertexAttribPointer(
            uv_attrib as gl::types::GLuint,
            2,
            gl::FLOAT,
            0,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            (3 * std::mem::size_of::<f32>()) as *const () as *const _,
        );
        gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
        gl.EnableVertexAttribArray(uv_attrib as gl::types::GLuint);

        // テクスチャ
        Texture::new(&gl, "res/tuku.png", 0);
    }

    Gl { gl }
}

impl Gl {
    pub fn draw_frame(&self, color: [f32; 4], pos: [f64; 2]) {
        unsafe {
            println!("pos: {:?}", pos);

            self.gl.ClearColor(color[0], color[1], color[2], color[3]);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);

            self.gl.MatrixMode(gl::PROJECTION); //投影変換モードへ
            self.gl.LoadIdentity(); //投影変換の変換行列を単位行列で初期化
            self.gl.Ortho(-1.0, 1.0, -1.0, 1.0, 1.0, -1.0); //各軸-1.0～1.0で囲まれる立方体の範囲を並行投影
        
            self.gl.MatrixMode(gl::MODELVIEW); //視野変換・モデリング変換モードへ
            self.gl.LoadIdentity(); //視野変換・モデリング変換の変換行列を単位行列で初期化
        
            self.gl.PushMatrix();
            self.gl.Translated(pos[0] / 400.0, pos[1] / -400.0, 0.0);
            self.gl.DrawElements(gl::TRIANGLES, INDEX_DATA.len() as i32, gl::UNSIGNED_BYTE, std::ptr::null());
            self.gl.PopMatrix();
        }
    }
}

#[rustfmt::skip]
static INDEX_DATA: [u8; 6] = [
    0, 1, 2, 0, 3, 2,
];

#[rustfmt::skip]
static VERTEX_DATA: [f32; 20] = [
    -0.5, -0.5,  0.0,  0.0,  0.0,
    -0.5,  0.5,  0.0,  1.0,  0.0,
     0.5,  0.5,  0.0,  1.0,  1.0,
     0.5, -0.5,  0.0,  0.0,  1.0,
];

const VS_SRC: &'static [u8] = b"
#version 410 compatibility
in vec4 pos;
in vec2 tex_coord;

out vec2 texture_coord;

void main()
{
    gl_Position = gl_ModelViewProjectionMatrix * pos;
    //gl_Position = pos;
    texture_coord = tex_coord;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 410 compatibility
out vec4 FragColor;

in vec2 texture_coord;

uniform sampler2D texture0;

void main()
{
    FragColor = texture(texture0, texture_coord);
    //FragColor = vec4(texture_coord.x, texture_coord.y, 0.0, 1.0);
}
\0";

use gl;
use gl::types::*;

use std::{mem, ptr, str};
use std::ffi::CString;

fn check_gl_error() {
    unsafe {
        match gl::GetError() {
            gl::NO_ERROR => {},
            gl::INVALID_ENUM => panic!("INVALID_ENUM"),
            gl::INVALID_VALUE => panic!("INVALID_VALUE"),
            gl::INVALID_OPERATION => panic!("INVALID_OPERATION"),
            gl::INVALID_FRAMEBUFFER_OPERATION => panic!("INVALID_FRAMEBUFFER_OPERATION"),
            gl::OUT_OF_MEMORY => panic!("OUT_OF_MEMORY"),
            gl::STACK_UNDERFLOW => panic!("STACK_UNDERFLOW"),
            gl::STACK_OVERFLOW => panic!("STACK_OVERFLOW"),
            _ => unreachable!(),
        }
    }
}

pub fn init_geometry() {
    unsafe {
        let mut geometry = 0;
        gl::CreateBuffers(1, &mut geometry);
        gl::BindBuffer(gl::ARRAY_BUFFER, geometry);

        let data: &[f32] = &[
            -1.0, -1.0,  0.0, 2.0,
             3.0, -1.0,  2.0, 2.0,
            -1.0,  3.0,  0.0, 0.0,
        ];
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * mem::size_of::<f32>()) as isize,
            data.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        check_gl_error();
    }
}

fn load_shader(kind: GLuint, source: &str) -> GLuint {
    let scr_c = CString::new(source).unwrap();
    unsafe {
        let shader = gl::CreateShader(kind);
        gl::ShaderSource(shader, 1, &scr_c.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar
            );

            panic!("{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }

        check_gl_error();

        shader
    }
}

#[derive(Debug)]
pub struct Program {
    program: GLuint,
    position: GLuint,
    uv_sampler: GLuint,
    board_sampler: GLuint,
}

pub fn load_program(frag: &str) -> Program {
    unsafe {
        let program = gl::CreateProgram();

        gl::AttachShader(program, load_shader(gl::VERTEX_SHADER, "
            attribute vec4 position;
            varying vec2 UV;

            void main() {
                UV = position.zw;
                gl_Position = vec4(position.xy, 0.0, 1.0);
            }
        "));

        gl::AttachShader(program, load_shader(gl::FRAGMENT_SHADER, frag));

        gl::LinkProgram(program);

        check_gl_error();

        Program {
            program,
            position: gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr()) as GLuint,
            uv_sampler: gl::GetUniformLocation(program, CString::new("uvSampler").unwrap().as_ptr()) as GLuint,
            board_sampler: gl::GetUniformLocation(program, CString::new("boardSampler").unwrap().as_ptr()) as GLuint,
        }
    }
}

pub fn make_texture(tex_size: usize) -> GLuint {
    unsafe {
        let mut texture = 0;
        gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0, gl::RGBA as GLint,
            tex_size as GLsizei,
            tex_size as GLsizei,
            0, gl::RGBA,
            gl::UNSIGNED_BYTE,
            ptr::null(),
        );

        texture
    }
}

/// Update a texture's data buffer
pub fn update_tex(texture: GLuint, tex_size: usize, data: Vec<u8>) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0, gl::RGBA as GLint,
            tex_size as GLsizei,
            tex_size as GLsizei,
            0, gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _,
        );

        check_gl_error();
    }
}

/// Write an UV / color texture pair to the screen
pub fn blit(program: &Program, (uv, board): (GLuint, GLuint), screen_size: u32) {
    let &Program { program, position, uv_sampler, board_sampler } = program;

    unsafe {
        gl::Viewport(0, 0, screen_size as GLint, screen_size as GLint);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        gl::UseProgram(program);
        gl::EnableVertexAttribArray(position);
        gl::VertexAttribPointer(position, 4, gl::FLOAT, 0, 0, ptr::null());

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, uv);
        gl::Uniform1i(uv_sampler as GLint, 0);
        check_gl_error();

        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, board);
        gl::Uniform1i(board_sampler as GLint, 1);
        check_gl_error();

        gl::DrawArrays(gl::TRIANGLES, 0, 3);

        check_gl_error();
    }
}

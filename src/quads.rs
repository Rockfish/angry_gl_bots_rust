use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::{gl, null, SIZE_OF_FLOAT};

#[rustfmt::skip]
const UNIT_SQUARE: [f32; 30] = [
    -1.0, -1.0, 0.0, 0.0, 0.0,
     1.0, -1.0, 0.0, 1.0, 0.0,
     1.0,  1.0, 0.0, 1.0, 1.0,
    -1.0, -1.0, 0.0, 0.0, 0.0,
     1.0,  1.0, 0.0, 1.0, 1.0,
    -1.0,  1.0, 0.0, 0.0, 1.0,
];

#[rustfmt::skip]
const MORE_OBNOXIOUS_QUAD: [f32; 30] = [
    -1.0, -1.0, -0.9, 0.0, 0.0,
     1.0, -1.0, -0.9, 1.0, 0.0,
     1.0,  1.0, -0.9, 1.0, 1.0,
    -1.0, -1.0, -0.9, 0.0, 0.0,
     1.0,  1.0, -0.9, 1.0, 1.0,
    -1.0,  1.0, -0.9, 0.0, 1.0,
];

#[rustfmt::skip]
const OBNOXIOUS_QUAD: [f32; 30] = [
    0.5, 0.5, -0.9, 0.0, 0.0,
    1.0, 0.5, -0.9, 1.0, 0.0,
    1.0, 1.0, -0.9, 1.0, 1.0,
    0.5, 0.5, -0.9, 0.0, 0.0,
    1.0, 1.0, -0.9, 1.0, 1.0,
    0.5, 1.0, -0.9, 0.0, 1.0,
];

pub fn create_obnoxious_quad_vao() -> GLuint {
    let mut obnoxious_quad_vao: GLuint = 0;
    let mut obnoxious_quad_vbo: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut obnoxious_quad_vao);
        gl::GenBuffers(1, &mut obnoxious_quad_vbo);
        gl::BindVertexArray(obnoxious_quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, obnoxious_quad_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (OBNOXIOUS_QUAD.len() * SIZE_OF_FLOAT) as GLsizeiptr,
            OBNOXIOUS_QUAD.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
    }
    obnoxious_quad_vao
}

pub fn create_unit_square_vao() -> GLuint {
    let mut unit_square_vao: GLuint = 0;
    let mut unit_square_vbo: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut unit_square_vao);
        gl::GenBuffers(1, &mut unit_square_vbo);
        gl::BindVertexArray(unit_square_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, unit_square_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (UNIT_SQUARE.len() * SIZE_OF_FLOAT) as GLsizeiptr,
            UNIT_SQUARE.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
    }
    unit_square_vao
}

pub fn create_more_obnoxious_quad_vao() -> GLuint {
    let mut more_obnoxious_quad_vao: GLuint = 0;
    let mut more_obnoxious_quad_vbo: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut more_obnoxious_quad_vao);
        gl::GenBuffers(1, &mut more_obnoxious_quad_vbo);
        gl::BindVertexArray(more_obnoxious_quad_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, more_obnoxious_quad_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (MORE_OBNOXIOUS_QUAD.len() * SIZE_OF_FLOAT) as GLsizeiptr,
            MORE_OBNOXIOUS_QUAD.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, null!());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
    }
    more_obnoxious_quad_vao
}

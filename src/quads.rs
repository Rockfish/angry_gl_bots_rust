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

pub fn create_obnoxiousQuadVAO() -> GLuint {
    let mut obnoxiousQuadVAO: GLuint = 0;
    let mut obnoxiousQuadVBO: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut obnoxiousQuadVAO);
        gl::GenBuffers(1, &mut obnoxiousQuadVBO);
        gl::BindVertexArray(obnoxiousQuadVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, obnoxiousQuadVBO);
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
    obnoxiousQuadVAO
}

pub fn create_unitSquareVAO() -> GLuint {
    let mut unitSquareVAO: GLuint = 0;
    let mut unitSquareVBO: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut unitSquareVAO);
        gl::GenBuffers(1, &mut unitSquareVBO);
        gl::BindVertexArray(unitSquareVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, unitSquareVBO);
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
    unitSquareVAO
}

pub fn create_moreObnoxiousQuadVAO() -> GLuint {
    let mut moreObnoxiousQuadVAO: GLuint = 0;
    let mut moreObnoxiousQuadVBO: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut moreObnoxiousQuadVAO);
        gl::GenBuffers(1, &mut moreObnoxiousQuadVBO);
        gl::BindVertexArray(moreObnoxiousQuadVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, moreObnoxiousQuadVBO);
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
    moreObnoxiousQuadVAO
}

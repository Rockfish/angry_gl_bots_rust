use small_gl_core::gl::{GLfloat, GLint, GLuint, GLvoid};
use small_gl_core::{gl, null};

const BLUR_SCALE: i32 = 1; // 2.0;

pub struct FrameBuffer {
    pub framebuffer_id: u32, // framebuffer object
    pub texture_id: u32,     // texture object
}

pub fn create_depth_map_fbo() -> FrameBuffer {
    let mut depth_map_fbo: GLuint = 0;
    let mut depth_map_texture: GLuint = 0;

    let SHADOW_WIDTH = 6 * 1024;
    let SHADOW_HEIGHT = 6 * 1024;
    let borderColor = [1.0f32, 1.0f32, 1.0f32, 1.0f32];

    unsafe {
        gl::GenFramebuffers(1, &mut depth_map_fbo);
        gl::GenTextures(1, &mut depth_map_texture);

        gl::BindTexture(gl::TEXTURE_2D, depth_map_texture);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::DEPTH_COMPONENT as GLint,
            SHADOW_WIDTH,
            SHADOW_HEIGHT,
            0,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);

        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, borderColor.as_ptr());
        gl::BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map_texture, 0);
        gl::DrawBuffer(gl::NONE);
        gl::ReadBuffer(gl::NONE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    FrameBuffer {
        framebuffer_id: depth_map_fbo,
        texture_id: depth_map_texture,
    }
}

pub fn create_emission_fbo(viewportWidth: i32, viewportHeight: i32) -> FrameBuffer {
    let mut emission_fbo: GLuint = 0;
    let mut emission_texture: GLuint = 0;

    unsafe {
        gl::GenFramebuffers(1, &mut emission_fbo);
        gl::GenTextures(1, &mut emission_texture);

        gl::BindFramebuffer(gl::FRAMEBUFFER, emission_fbo);
        gl::BindTexture(gl::TEXTURE_2D, emission_texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            viewportWidth,
            viewportHeight,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
        let borderColor2: [GLfloat; 4] = [0.0, 0.0, 0.0, 0.0];
        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, borderColor2.as_ptr());
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, emission_texture, 0);

        let mut rbo: GLuint = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT16, viewportWidth, viewportHeight);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rbo);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: emission_fbo,
        texture_id: emission_texture,
    }
}

pub fn create_scene_fbo(viewportWidth: i32, viewportHeight: i32) -> FrameBuffer {
    let mut sceneRenderFBO: GLuint = 0;
    let mut sceneBuffer: GLuint = 0;

    unsafe {
        gl::GenFramebuffers(1, &mut sceneRenderFBO);
        gl::GenTextures(1, &mut sceneBuffer);

        gl::BindFramebuffer(gl::FRAMEBUFFER, sceneRenderFBO);
        gl::BindTexture(gl::TEXTURE_2D, sceneBuffer);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            viewportWidth,
            viewportHeight,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, sceneBuffer, 0);

        let mut rbo: GLuint = 0;

        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, viewportWidth, viewportHeight);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Frame buffer not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: sceneRenderFBO,
        texture_id: sceneBuffer,
    }
}

pub fn create_horizontal_blur_fbo(viewportWidth: i32, viewportHeight: i32) -> FrameBuffer {
    let mut horzBlurFBO: GLuint = 0;
    let mut horzBlurBuffer: GLuint = 0;

    unsafe {
        gl::GenFramebuffers(1, &mut horzBlurFBO);
        gl::GenTextures(1, &mut horzBlurBuffer);

        gl::BindFramebuffer(gl::FRAMEBUFFER, horzBlurFBO);
        gl::BindTexture(gl::TEXTURE_2D, horzBlurBuffer);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            viewportWidth / BLUR_SCALE,
            viewportHeight / BLUR_SCALE,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, horzBlurBuffer, 0);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Frame buffer not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: horzBlurFBO,
        texture_id: horzBlurBuffer,
    }
}

pub fn create_vertical_blur_fbo(viewportWidth: i32, viewportHeight: i32) -> FrameBuffer {
    let mut vertBlurFBO: GLuint = 0;
    let mut vertBlurBuffer: GLuint = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut vertBlurFBO);
        gl::GenTextures(1, &mut vertBlurBuffer);

        gl::BindFramebuffer(gl::FRAMEBUFFER, vertBlurFBO);
        gl::BindTexture(gl::TEXTURE_2D, vertBlurBuffer);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            viewportWidth / BLUR_SCALE,
            viewportHeight / BLUR_SCALE,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, vertBlurBuffer, 0);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Frame buffer not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: vertBlurFBO,
        texture_id: vertBlurBuffer,
    }
}

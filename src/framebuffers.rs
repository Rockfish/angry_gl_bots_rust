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

    let shadow_width = 6 * 1024;
    let shadow_height = 6 * 1024;
    let border_color = [1.0f32, 1.0f32, 1.0f32, 1.0f32];

    unsafe {
        gl::GenFramebuffers(1, &mut depth_map_fbo);
        gl::GenTextures(1, &mut depth_map_texture);

        gl::BindTexture(gl::TEXTURE_2D, depth_map_texture);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::DEPTH_COMPONENT as GLint,
            shadow_width,
            shadow_height,
            0,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);

        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr());
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

pub fn create_emission_fbo(viewport_width: i32, viewport_height: i32) -> FrameBuffer {
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
            viewport_width,
            viewport_height,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
        let border_color2: [GLfloat; 4] = [0.0, 0.0, 0.0, 0.0];
        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_color2.as_ptr());
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, emission_texture, 0);

        let mut rbo: GLuint = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT16, viewport_width, viewport_height);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rbo);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: emission_fbo,
        texture_id: emission_texture,
    }
}

pub fn create_scene_fbo(viewport_width: i32, viewport_height: i32) -> FrameBuffer {
    let mut scene_render_fbo: GLuint = 0;
    let mut scene_buffer: GLuint = 0;

    unsafe {
        gl::GenFramebuffers(1, &mut scene_render_fbo);
        gl::GenTextures(1, &mut scene_buffer);

        gl::BindFramebuffer(gl::FRAMEBUFFER, scene_render_fbo);
        gl::BindTexture(gl::TEXTURE_2D, scene_buffer);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            viewport_width,
            viewport_height,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, scene_buffer, 0);

        let mut rbo: GLuint = 0;

        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, viewport_width, viewport_height);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Frame buffer not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: scene_render_fbo,
        texture_id: scene_buffer,
    }
}

pub fn create_horizontal_blur_fbo(viewport_width: i32, viewport_height: i32) -> FrameBuffer {
    let mut horz_blur_fbo: GLuint = 0;
    let mut horz_blur_buffer: GLuint = 0;

    unsafe {
        gl::GenFramebuffers(1, &mut horz_blur_fbo);
        gl::GenTextures(1, &mut horz_blur_buffer);

        gl::BindFramebuffer(gl::FRAMEBUFFER, horz_blur_fbo);
        gl::BindTexture(gl::TEXTURE_2D, horz_blur_buffer);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            viewport_width / BLUR_SCALE,
            viewport_height / BLUR_SCALE,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, horz_blur_buffer, 0);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Frame buffer not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: horz_blur_fbo,
        texture_id: horz_blur_buffer,
    }
}

pub fn create_vertical_blur_fbo(viewport_width: i32, viewport_height: i32) -> FrameBuffer {
    let mut vert_blur_fbo: GLuint = 0;
    let mut vert_blur_buffer: GLuint = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut vert_blur_fbo);
        gl::GenTextures(1, &mut vert_blur_buffer);

        gl::BindFramebuffer(gl::FRAMEBUFFER, vert_blur_fbo);
        gl::BindTexture(gl::TEXTURE_2D, vert_blur_buffer);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            viewport_width / BLUR_SCALE,
            viewport_height / BLUR_SCALE,
            0,
            gl::RGB,
            gl::FLOAT,
            null!(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, vert_blur_buffer, 0);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Frame buffer not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    FrameBuffer {
        framebuffer_id: vert_blur_fbo,
        texture_id: vert_blur_buffer,
    }
}

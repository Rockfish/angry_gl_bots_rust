use small_gl_core::{gl, null};
use small_gl_core::gl::{GLint, GLuint, GLvoid};

pub fn setup_depth_map() -> GLuint {
    let mut depthMapFBO: GLuint = 0;
    let mut depthMap: GLuint = 0;
    unsafe {
        // gl::ActiveTexture(gl::TEXTURE0 + texUnit_shadowMap);
        gl::GenFramebuffers(1, &mut depthMapFBO);
        let SHADOW_WIDTH = 6 * 1024;
        let SHADOW_HEIGHT = 6 * 1024;
        gl::GenTextures(1, &mut depthMap);
        gl::BindTexture(gl::TEXTURE_2D, depthMap);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as GLint, SHADOW_WIDTH, SHADOW_HEIGHT, 0, gl::DEPTH_COMPONENT, gl::FLOAT, null!());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as GLint);
        let borderColor = [1.0f32, 1.0f32, 1.0f32, 1.0f32 ];
        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, borderColor.as_ptr());
        gl::BindFramebuffer(gl::FRAMEBUFFER, depthMapFBO);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depthMap, 0);
        gl::DrawBuffer(gl::NONE);
        gl::ReadBuffer(gl::NONE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    depthMapFBO
}

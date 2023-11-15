#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

mod capsule;
mod enemy;
mod geom;
mod sprite_sheet;
mod bullet_store;

extern crate glfw;

use crate::enemy::{Enemy, ENEMY_COLLIDER};
use crate::geom::distanceBetweenPointAndLineSegment;
use glam::{vec2, vec3, Mat4, Vec3, Vec2};
use glfw::{Action, Context, Key};
use log::error;
use small_gl_core::camera::{Camera, CameraMovement};
use small_gl_core::{gl, SIZE_OF_FLOAT};
use small_gl_core::mesh::{Color, Mesh};
use small_gl_core::model::{Model, ModelBuilder};
use small_gl_core::shader::Shader;
use small_gl_core::texture::{Texture, TextureConfig, TextureFilter, TextureType};
use std::path::PathBuf;
use std::rc::Rc;
use std::f32::consts::PI;
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use crate::bullet_store::BulletStore;
use crate::sprite_sheet::SpriteSheet;

const SCR_WIDTH: f32 = 1000.0;
const SCR_HEIGHT: f32 = 800.0;

const parallelism: i32 = 4;

// Viewport
const viewportWidth: i32 = 1500;
const viewportHeight: i32 = 1000;

// Texture units
const texUnit_playerDiffuse: i32 = 0;
const texUnit_gunDiffuse: i32 = 1;
const texUnit_floorDiffuse: i32 = 2;
const texUnit_wigglyBoi: i32 = 3;
const texUnit_bullet: i32 = 4;
const texUnit_floorNormal: i32 = 5;
const texUnit_playerNormal: i32 = 6;
const texUnit_gunNormal: i32 = 7;
const texUnit_shadowMap: i32 = 8;
const texUnit_emissionFBO: i32 = 9;
const texUnit_playerEmission: i32 = 10;
const texUnit_gunEmission: i32 = 11;
const texUnit_scene: i32 = 12;
const texUnit_horzBlur: i32 = 13;
const texUnit_vertBlur: i32 = 14;
const texUnit_impactSpriteSheet: i32 = 15;
const texUnit_muzzleFlashSpriteSheet: i32 = 16;
const texUnit_floorSpec: i32 = 18;
const texUnit_playerSpec: i32 = 19;
const texUnit_gunSpec: i32 = 20;

// Player
const fireInterval: f32 = 0.1; // seconds
const spreadAmount: i32 = 20;
const playerSpeed: f32 = 1.5;
const playerCollisionRadius: f32 = 0.35;

// Models
const playerModelScale: f32 = 0.0044;
const playerModelGunHeight: f32 = 120.0; // un-scaled
const playerModelGunMuzzleOffset: f32 = 100.0; // un-scaled
const monsterY: f32 = playerModelScale * playerModelGunHeight;

// Lighting
const lightFactor: f32 = 0.8;
const nonBlue: f32 = 0.9;

const floorLightFactor: f32 = 0.35;
const floorNonBlue: f32 = 0.7;

// Enemies
const monsterSpeed: f32 = 0.6;

#[rustfmt::skip]
const unitSquare: [f32; 30] = [
    -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, -1.0, 0.0,
    0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0,
];

#[rustfmt::skip]
const moreObnoxiousQuad: [f32; 30] = [
    -1.0, -1.0, -0.9, 0.0, 0.0, 1.0, -1.0, -0.9, 1.0, 0.0, 1.0, 1.0, -0.9, 1.0, 1.0, -1.0, -1.0,
    -0.9, 0.0, 0.0, 1.0, 1.0, -0.9, 1.0, 1.0, -1.0, 1.0, -0.9, 0.0, 1.0,
];

#[rustfmt::skip]
const obnoxiousQuad: [f32; 30] = [
    0.5, 0.5, -0.9, 0.0, 0.0, 1.0, 0.5, -0.9, 1.0, 0.0, 1.0, 1.0, -0.9, 1.0, 1.0, 0.5, 0.5, -0.9,
    0.0, 0.0, 1.0, 1.0, -0.9, 1.0, 1.0, 0.5, 1.0, -0.9, 0.0, 1.0,
];

struct Player {
    lastFireTime: f32,
    isTryingToFire: bool,
    isAlive: bool,
    aimTheta: f32,
    playerPosition: Vec3,
    playerMovementDir: Vec2,
}

struct State {
    camera: Camera,
    delta_time: f32,
    last_frame: f32,
    first_mouse: bool,
    last_x: f32,
    last_y: f32,
    player: Player,
    enemies: Vec<Enemy>,
}

fn error_callback(err: glfw::Error, description: String) {
    error!("GLFW error {:?}: {:?}", err, description);
}

fn main() {
    let mut glfw = glfw::init(error_callback).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH as u32,
            SCR_HEIGHT as u32,
            "LearnOpenGL",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_all_polling(true);
    window.make_current();

    gl::load(|e| glfw.get_proc_address_raw(e) as *const std::os::raw::c_void);

    // Mouse input
    let mut mouseClipX = 0.0f32;
    let mut mouseClipY = 0.0f32;

    // Lighting
    let lightDir: Vec3 = vec3(-0.8, 0.0, -1.0).normalize_or_zero();
    let playerLightDir: Vec3 = vec3(-1.0, -1.0, -1.0).normalize_or_zero();
    let lightColor: Vec3 = lightFactor * 1.0 * vec3(nonBlue * 0.406, nonBlue * 0.723, 1.0);
    // const lightColor: Vec3 = lightFactor * 1.0 * vec3(0.406, 0.723, 1.0);
    let floorLightColor: Vec3 =
        floorLightFactor * 1.0 * vec3(floorNonBlue * 0.406, floorNonBlue * 0.723, 1.0);
    let floorAmbientColor: Vec3 =
        floorLightFactor * 0.50 * vec3(floorNonBlue * 0.7, floorNonBlue * 0.7, 0.7);
    let ambientColor: Vec3 = lightFactor * 0.10 * vec3(nonBlue * 0.7, nonBlue * 0.7, 0.7);

    let muzzlePointLightColor = vec3(1.0, 0.2, 0.0);

    // Camera
    let cameraFollowVec = vec3(-4.0, 4.3, 0.0);
    let cameraUp = vec3(0.0, 1.0, 0.0);

    // perspective setting
    let camera = Camera::camera_vec3_up_yaw_pitch(
        // vec3(400.0, 400.0, 700.0), for current x,y world
        vec3(0.0, 170.0, 500.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
        -20.0,
    );

    // Player
    let player = Player {
        lastFireTime: 0.0f32,
        isTryingToFire: false,
        isAlive: true,
        aimTheta: 0.0f32,
        playerPosition: vec3(0.0, 0.0, 0.0),
        playerMovementDir: vec2(0.0, 0.0),
    };

    let mut state = State {
        camera,
        delta_time: 0.0,
        last_frame: 0.0,
        first_mouse: true,
        last_x: SCR_WIDTH / 2.0,
        last_y: SCR_HEIGHT / 2.0,
        player,
        enemies: vec![],
    };


    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        let mut obnoxiousQuadVAO: GLuint = 0;
        let mut obnoxiousQuadVBO: GLuint = 0;
        gl::GenVertexArrays(1, &mut obnoxiousQuadVAO);
        gl::GenBuffers(1, &mut obnoxiousQuadVBO);
        gl::BindVertexArray(obnoxiousQuadVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, obnoxiousQuadVBO);
        gl::BufferData(gl::ARRAY_BUFFER, (obnoxiousQuad.len() * SIZE_OF_FLOAT) as GLsizeiptr, obnoxiousQuad.as_ptr() as *const GLvoid, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);


        let mut unitSquareVAO: GLuint = 0;
        let mut unitSquareVBO: GLuint = 0;
        gl::GenVertexArrays(1, &mut unitSquareVAO);
        gl::GenBuffers(1, &mut unitSquareVBO);
        gl::BindVertexArray(unitSquareVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, unitSquareVBO);
        gl::BufferData(gl::ARRAY_BUFFER, (unitSquare.len() * SIZE_OF_FLOAT) as GLsizeiptr, unitSquare.as_ptr() as *const GLvoid, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);

        let mut moreObnoxiousQuadVAO: GLuint = 0;
        let mut moreObnoxiousQuadVBO: GLuint = 0;
        gl::GenVertexArrays(1, &mut moreObnoxiousQuadVAO);
        gl::GenBuffers(1, &mut moreObnoxiousQuadVBO);
        gl::BindVertexArray(moreObnoxiousQuadVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, moreObnoxiousQuadVBO);
        gl::BufferData(gl::ARRAY_BUFFER, (moreObnoxiousQuad.len() * SIZE_OF_FLOAT) as GLsizeiptr, moreObnoxiousQuad.as_ptr() as *const GLvoid, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
    }

    println!("Loading assets");

    let blurShader = Shader::new("assets/shaders/basicer_shader.vert", "assets/shaders/blur_shader.frag").unwrap();
    let basicerShader = Shader::new("assets/shaders/basicer_shader.vert", "assets/shaders/basicer_shader.frag").unwrap();
    let sceneDrawShader = Shader::new("assets/shaders/basicer_shader.vert", "assets/shaders/texture_merge_shader.frag").unwrap();
    let simpleDepthShader = Shader::new("assets/shaders/depth_shader.vert", "assets/shaders/depth_shader.frag").unwrap();

    simpleDepthShader.use_shader();
    let lsml = simpleDepthShader.get_uniform_location("lightSpaceMatrix");

    let playerShader = Shader::new("assets/shaders/player_shader.vert", "assets/shaders/player_shader.frag").unwrap();
    playerShader.use_shader();

    let playerLightSpaceMatrixLocation = playerShader.get_uniform_location( "lightSpaceMatrix");
    playerShader.set_vec3("directionLight.dir", &playerLightDir);
    playerShader.set_vec3("directionLight.color", &lightColor);
    playerShader.set_vec3("ambient", &ambientColor);
    playerShader.set_int("texture_spec", texUnit_playerSpec);

    // logTimeSince("shaders loaded ", appStart);

    let bulletImpactSpritesheet = SpriteSheet::new(texUnit_impactSpriteSheet, 11, 0.05);
    let muzzleFlashImpactSpritesheet = SpriteSheet::new(texUnit_muzzleFlashSpriteSheet, 6, 0.05);

    let bulletStore = BulletStore::initialize_buffer_and_create(); // &threadPool);

    let wigglyShader = Shader::new("assets/shaders/wiggly_shader.vert", "assets/shaders/player_shader.frag").unwrap();
    let wigglyBoi = ModelBuilder::new("wigglyBoi", Rc::new(wigglyShader), "assets/models/eel_dog/EelDog.FBX");

    // render loop
    while !window.should_close() {
        let current_time = glfw.get_time() as f32;
        state.delta_time = current_time - state.last_frame;
        state.last_frame = current_time;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        unsafe {
            gl::ClearColor(0.0, 0.02, 0.45, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let view = state.camera.get_view_matrix();
        let projection = Mat4::perspective_rh_gl(
            state.camera.zoom.to_radians(),
            SCR_WIDTH / SCR_HEIGHT,
            0.1,
            2000.0,
        );

        window.swap_buffers();
    }
}

//
// GLFW maps callbacks to events.
//
fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, state: &mut State) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(width, height) => {
            framebuffer_size_event(window, width, height);
        }
        glfw::WindowEvent::Key(Key::W, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Forward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Backward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::A, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Left, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::D, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Right, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Q, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Up, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, _) => {
            state
                .camera
                .process_keyboard(CameraMovement::Down, state.delta_time);
        }
        glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
        glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
        _evt => {
            // println!("WindowEvent: {:?}", _evt);
        }
    }
}

// glfw: whenever the window size changed (by OS or user resize) this event fires.
// ---------------------------------------------------------------------------------------------
fn framebuffer_size_event(_window: &mut glfw::Window, width: i32, height: i32) {
    // make sure the viewport matches the new window dimensions; note that width and
    // height will be significantly larger than specified on retina displays.
    // println!("Framebuffer size: {}, {}", width, height);
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

fn mouse_handler(state: &mut State, xposIn: f64, yposIn: f64) {
    let xpos = xposIn as f32;
    let ypos = yposIn as f32;

    if state.first_mouse {
        state.last_x = xpos;
        state.last_y = ypos;
        state.first_mouse = false;
    }

    let xoffset = xpos - state.last_x;
    let yoffset = state.last_y - ypos; // reversed since y-coordinates go from bottom to top

    state.last_x = xpos;
    state.last_y = ypos;

    state.camera.process_mouse_movement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.camera.process_mouse_scroll(yoffset as f32);
}

fn chasePlayer(state: &mut State) {
    let playerCollisionPosition = vec3(state.player.playerPosition.x, monsterY, state.player.playerPosition.z);
    for enemy in state.enemies.iter_mut() {
        let mut dir = state.player.playerPosition - enemy.position;
        dir.y = 0.0;
        enemy.dir = dir.normalize_or_zero();
        enemy.position += enemy.dir * state.delta_time * monsterSpeed;
        if state.player.isAlive {
            let p1 = enemy.position - enemy.dir * (ENEMY_COLLIDER.height / 2.0);
            let p2 = enemy.position + enemy.dir * (ENEMY_COLLIDER.height / 2.0);
            let dist = distanceBetweenPointAndLineSegment(&playerCollisionPosition, &p1, &p2);
            if dist <= (playerCollisionRadius + ENEMY_COLLIDER.radius) {
                println!("GOTTEM!");
                state.player.isAlive = false;
                state.player.playerMovementDir = vec2(0.0, 0.0);
            }
        }
    }
}

fn drawWigglyBois(wigglyBoi: &Model, state: &mut State) {

    wigglyBoi.shader.use_shader();
    wigglyBoi.shader.set_vec3("nosePos", &vec3(1.0, monsterY, -2.0));

    // TODO optimise (multithread, instancing, SOA, etc..)
    for e in state.enemies.iter_mut() {
        let monsterTheta = (e.dir.x / e.dir.z).atan() + (if e.dir.z < 0.0 { 0.0 } else { PI });

        let mut model_transform = Mat4::from_translation(e.position);
        model_transform *= Mat4::from_scale(Vec3::splat(0.01));
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monsterTheta);
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
        model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

        let mut rot_only = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monsterTheta);
        rot_only = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
        rot_only = Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

        wigglyBoi.shader.set_mat4("aimRot", &rot_only);
        wigglyBoi.shader.set_mat4("model", &model_transform);

        wigglyBoi.render();
    }
}
extern crate nalgebra_glm as glm;
extern crate gl;

use std::{
    ptr,
};
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod util;
mod gl_utils;

use gl_utils::{
    triangle::Triangle,
    bindable::Bindable, shaders::program::ProgramBuilder, camera::{VecDir, CameraBuilder}
};

use glutin::event::{Event, WindowEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// Using Triangle abstraction (See gl_utils::triangle)

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    
    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Send a copy of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the renderin thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());
        }

        // == // Set up your VAO here
        // opengl goes from -1 to 1
        let mut pos: f32 = -1.0;
        let stride: f32 = 2.0 / 5.0;
        let mut vertices = Vec::<f32>::new();
        let mut indices = Vec::<u32>::new();
        for i in 0..5 {
            let mut new_triangle = vec![
                pos, -0.5, 0.0,
                pos + stride, -0.5, 0.0,
                pos + stride / 2.0, 0.5, 0.0
            ];

            vertices.append(&mut new_triangle);
            let index = i * 3;
            indices.append(&mut vec![index, index + 1, index + 2]);

            pos += stride;
        }

        // We could also inline hardcoded 5 triangles, but what's the fun in that ;)
        // Of course this would lead to easier code to read which is faster and objectively better ...
        let my_triangle = {
           Triangle::init(&vertices, &indices)
        };

        // Basic usage of shader helper
        let mut program = ProgramBuilder::new()
            .attach_file("assets/shaders/main.vert")
            .attach_file("assets/shaders/main.frag")
            .link();

            
        if let Err(e) = program.locate_uniform("elapsed") {
            eprint!("Failed to find elapsed, probably loading wrong shader. err: {}", e);
            return;
        };

        let mut camera = CameraBuilder::init()
            .projection(SCREEN_W as f32 / SCREEN_H as f32, 1.4, 0.1, 40.0)
            .transform(&glm::vec3(0.0, 0.0, -2.0))
            .move_speed(2.0)
            .build_and_attach_to_program(&mut program);

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W => camera.move_in_dir(VecDir::Forward, delta_time, &program),
                        VirtualKeyCode::S => camera.move_in_dir(VecDir::Backward, delta_time, &program),
                        VirtualKeyCode::A => camera.move_in_dir(VecDir::Left, delta_time, &program),
                        VirtualKeyCode::D => camera.move_in_dir(VecDir::Right, delta_time, &program),
                        VirtualKeyCode::Space => camera.move_in_dir(VecDir::Up, delta_time, &program),
                        VirtualKeyCode::LControl => camera.move_in_dir(VecDir::Down, delta_time, &program),
                        _ => { }
                    }
                }
            }

            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                my_triangle.bind();
                gl::UseProgram(program.program_id);

                if let Err(e) = program.set_uniform1("elapsed", elapsed, gl::Uniform1f) {
                    eprintln!("{}", e)
                };
       
                gl::DrawElements(
                    gl::TRIANGLES,
                    my_triangle.count,
                    gl::UNSIGNED_INT,
                    std::ptr::null() 
                );

                gl::UseProgram(0);
                my_triangle.unbind();
            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => { }
                }
            },
            _ => { }
        }
    });
}

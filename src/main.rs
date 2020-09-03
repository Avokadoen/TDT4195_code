extern crate nalgebra_glm as glm;
extern crate gl;

use glutin::event::KeyboardInput;
use std::{
    ptr,
};
use std::thread;
use std::sync::{Mutex, Arc, RwLock, mpsc};

mod util;
mod gl_utils;

use gl_utils::{
    geometric_object::GeometricObject,
    bindable::Bindable, 
    shaders::program::ProgramBuilder, 
    camera::{VecDir, CameraBuilder}, obj_loader::load_and_parse_obj
};

use glutin::event::{
    Event, 
    WindowEvent, 
    ElementState::{Pressed, Released}, 
    VirtualKeyCode::{self, *}, 
    DeviceEvent
};

use glutin::{window::Fullscreen, event_loop::ControlFlow};

enum InputEvent {
    Key(KeyboardInput),
    Mouse((f64, f64))
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_maximized(false)
        .with_always_on_top(true);
        //.with_fullscreen(Some(Fullscreen::Borderless(el.primary_monitor())));

    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);

    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    if let Err(e) = windowed_context.window().set_cursor_grab(true) {
        panic!("Error grabbing mouse, e: {}", e);
    }
    windowed_context.window().set_cursor_visible(false);
  
    let (tx, rx) = mpsc::channel::<InputEvent>();

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        let sf = windowed_context.window().scale_factor();
        let screen_dimensions = windowed_context.window().inner_size().to_logical::<f32>(sf);


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
        
        // We could also inline hardcoded 5 triangles, but what's the fun in that ;)
        // Of course this would lead to easier code to read which is faster and objectively better ...
        let my_triangle = {
            let parsed_obj = load_and_parse_obj("assets/objs/teapot.obj");
            match parsed_obj {
                Ok(o) => {
                    GeometricObject::init(&o.vertices, &o.faces) 
                },
                Err(e) => panic!("Failed to load obj, e: {}", e)
            }
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

        if let Err(e) = program.locate_uniform("transform") {
            eprint!("Failed to find transform, probably loading wrong shader. err: {}", e);
            return;
        };

        let transform = glm::scale(&glm::identity::<f32, glm::U4>(), &glm::vec3(0.01, 0.01, 0.01));
        if let Err(e) = program.set_uniform_matrix("transform", transform.as_ptr(), gl::UniformMatrix4fv) {
            eprintln!("Error occured while assigning transform, e: {}", e);
        }

        let mut camera = CameraBuilder::init()
            .projection(screen_dimensions.width / screen_dimensions.height, 1.4, 0.1, 40.0)
            .transform(&glm::vec3(0.0, 0.0, -2.0))
            .move_speed(2.0)
            .turn_sensitivity(0.2)
            .build_and_attach_to_program(&mut program);

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        // TODO: Virtual input abstraction for runtime settings
        let mut pressed_keys = Vec::<VirtualKeyCode>::with_capacity(10);

        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle changes in keyboard state
            rx.try_iter().for_each(|input_event| {
                match input_event {
                    InputEvent::Key(key_input) => {
                        match key_input.state {
                            Pressed => {
                                if let Some(code) = key_input.virtual_keycode {
                                    if let None = pressed_keys.iter().position(|x| *x == code) {
                                        pressed_keys.push(code);
                                    }
                                }
                            }
                            Released => {
                                if let Some(code) = key_input.virtual_keycode {
                                    if let Some(i) = pressed_keys.iter().position(|x| *x == code) {
                                        pressed_keys.swap_remove(i);
                                    }
                                }
                            }
                        }
                    }
                    InputEvent::Mouse(mouse_input) => {
                        camera.turn(mouse_input, delta_time, &program);
                    }
                }
            });

            // Handle keyboard input
            pressed_keys.iter().for_each(|key| {
                match key {
                    VirtualKeyCode::W => camera.move_in_dir(VecDir::Forward, delta_time, &program),
                    VirtualKeyCode::S => camera.move_in_dir(VecDir::Backward, delta_time, &program),
                    VirtualKeyCode::A => camera.move_in_dir(VecDir::Left, delta_time, &program),
                    VirtualKeyCode::D => camera.move_in_dir(VecDir::Right, delta_time, &program),
                    VirtualKeyCode::Space => camera.move_in_dir(VecDir::Up, delta_time, &program),
                    VirtualKeyCode::LControl => camera.move_in_dir(VecDir::Down, delta_time, &program),
                    _ => { }
                }
            });

            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

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
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // TODO:
                // WindowEvent::Resized(physical_size) => {
                    // windowed_context.resize(physical_size);
                // }
                // Send event to rendering thread
                WindowEvent::KeyboardInput { input, ..} => {
                    if let Err(e) = tx.send(InputEvent::Key(input)) {
                        eprintln!("Seems reciever has died, e: {}", e);
                    }

                    if let Some(Escape) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit;
                    }
                },
                _ => (),
            }
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion {delta} => {
                    if let Err(e) = tx.send(InputEvent::Mouse(delta)) {
                        eprintln!("Seems reciever has died, e: {}", e);
                    }
                },
                _ => { }
            }
            _ => { }
        }
    });
}

extern crate nalgebra_glm as glm;
extern crate gl;
extern crate tobj;

use glutin::event::KeyboardInput;
use std::{
    ptr,
};
use std::thread;
use std::{env, sync::{Arc, RwLock, mpsc}};

mod util;
mod gl_utils;

use gl_utils::{
    bindable::Bindable, 
    camera::{VecDir, CameraBuilder}, 
    geometric_object::GeometricObject, 
    mesh::Terrain, shaders::program::ProgramBuilder, 
    vertex_attributes::VerticesAttributesPair
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
    
    let wb = {
        let mut wb  = glutin::window::WindowBuilder::new()
            .with_title("Gloom-rs")
            .with_resizable(false)
            .with_always_on_top(true);

        let args: Vec<String> = env::args().collect();
        for arg in args.iter().skip(1) {
            match &arg[..] {
                "-f" | "-F" => {
                    wb = wb.with_maximized(true)
                    .with_fullscreen(Some(Fullscreen::Borderless(el.primary_monitor())));
                },
                "-h" => {
                    let h_command = "\n-h => 'display this information'";
                    let f_command = "\n-f | -F => 'fullscreen mode'"; // TODO: fov and mouse sense should be connected to this somehow
                    println!("Rendering toy code{}{}", h_command, f_command);
                    return;
                },
                c => eprintln!("Unknown command '{}'", c)
            }
        }

        wb
    };

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
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS); 

            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());
        }
        
        let terrain_geometry = {
            // TODO: utility in mesh to convert to attrib_pair vec
            let terrain = Terrain::load("assets/objs/lunarsurface.obj");
            let buffer_attrib_pairs = vec![
                VerticesAttributesPair::init(terrain.vertices, gl::FLOAT).add_attribute(0, 0, 3, 0),
                VerticesAttributesPair::init(terrain.normals, gl::FLOAT).add_attribute(1, 1, 3, 0),
                VerticesAttributesPair::init(terrain.colors, gl::FLOAT).add_attribute(2, 2, 4, 0),
            ];

            GeometricObject::init(&buffer_attrib_pairs, &terrain.indices, &vec![glm::Mat4::identity()])
        };

        // Basic usage of shader helper
        let mut terrain_program = ProgramBuilder::new()
            .attach_file("assets/shaders/terrain.vert")
            .attach_file("assets/shaders/terrain.frag")
            .link();

        let mut camera = CameraBuilder::init()
            .projection(screen_dimensions.width / screen_dimensions.height, 1.4, 0.1, 1000.0)
            .translation(&glm::vec3(0.0, 0.0, 0.0))
            .move_speed(2.0)
            .turn_sensitivity(0.2)
            .build_and_attach_to_program(&mut terrain_program);

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        // TODO: Virtual input abstraction for runtime settings
        // TODO: This can be an array instead of a Vec
        let mut pressed_keys = Vec::<VirtualKeyCode>::with_capacity(10);    
        let mut disable_turn = false;

        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            // let elapsed = now.duration_since(first_frame_time).as_secs_f32();
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
                        if !disable_turn {
                            camera.turn(mouse_input, delta_time, &terrain_program);
                        }
                    }
                }
            });

            // Handle keyboard input
            pressed_keys.iter().for_each(|key| {
                match key {
                    VirtualKeyCode::W => camera.move_in_dir(VecDir::Forward, delta_time, &terrain_program),
                    VirtualKeyCode::S => camera.move_in_dir(VecDir::Backward, delta_time, &terrain_program),
                    VirtualKeyCode::A => camera.move_in_dir(VecDir::Left, delta_time, &terrain_program),
                    VirtualKeyCode::D => camera.move_in_dir(VecDir::Right, delta_time, &terrain_program),
                    VirtualKeyCode::R => disable_turn = !disable_turn,
                    VirtualKeyCode::Space => camera.move_in_dir(VecDir::Up, delta_time, &terrain_program),
                    VirtualKeyCode::LControl => camera.move_in_dir(VecDir::Down, delta_time, &terrain_program),
                    _ => { }
                }
            });

            unsafe {
                gl::ClearColor(0.05, 0.05, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
                terrain_geometry.draw_all(&terrain_program);
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

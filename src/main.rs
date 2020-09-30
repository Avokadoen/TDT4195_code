extern crate nalgebra_glm as glm;
extern crate gl;
extern crate tobj;

use glutin::event::KeyboardInput;
use std::{
    thread,
    ptr,
    env, 
    sync::{Arc, RwLock, mpsc}
};

mod util;
mod gl_utils;

use gl_utils::{camera::{VecDir, CameraBuilder}, mesh::{Helicopter, Terrain}, scene_graph::SceneNode, shaders::program::ProgramBuilder};

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
            // gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());
        }
        
        // Basic usage of shader helper
        let program = ProgramBuilder::new()
            .attach_file("assets/shaders/main.vert")
            .attach_file("assets/shaders/main.frag")
            .link();

        let single_instance = vec![glm::Mat4::identity()];

        let terrain_geometry = {
            // TODO: utility in mesh to convert to attrib_pair vec
            let terrain = Terrain::load("assets/objs/lunarsurface.obj");
            terrain.into_geomtric_object(program.program_id, &single_instance)
        };

        let (body_geometry, main_rot_geometry, tail_rot_geometry, door_geometry) = {
            let h = Helicopter::load("assets/objs/helicopter.obj");
            // We dissect helicopter to make it easier to take ownership of each mesh
            (
                h.body.into_geomtric_object(program.program_id, &single_instance), 
                h.main_rotor.into_geomtric_object(program.program_id, &single_instance), 
                h.tail_rotor.into_geomtric_object(program.program_id, &single_instance), 
                h.door.into_geomtric_object(program.program_id, &single_instance)
            )
        };

        let mut scene_graph = SceneNode::new();

        let mut terrain_node = SceneNode::from_vao(terrain_geometry);
        scene_graph.add_child(&terrain_node);
        
        let mut helicopter_node = SceneNode::from_vao(body_geometry);
            
        let mut main_rot_node = SceneNode::from_vao(main_rot_geometry);
        helicopter_node.add_child(&main_rot_node);

        let mut tail_rot_node = SceneNode::from_vao(tail_rot_geometry);
        tail_rot_node.set_reference_point(glm::vec3(0.35,2.3,10.4));
        helicopter_node.add_child(&tail_rot_node);
        
        let door_node = SceneNode::from_vao(door_geometry);
        helicopter_node.add_child(&door_node);


            

        terrain_node.add_child(&helicopter_node);

        let mut camera = CameraBuilder::init()
            .projection(screen_dimensions.width / screen_dimensions.height, 1.4, 0.1, 1000.0)
            .translation(&glm::vec3(0.0, 0.0, 0.0))
            .move_speed(14.0)
            .turn_sensitivity(0.2)
            .build_and_attach_to_programs(vec![program]);

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

            main_rot_node.euler_rotation.y = main_rot_node.euler_rotation.y % 360.0; // Avoid floating point errors
            main_rot_node.euler_rotation.y += delta_time * 1080.0; // rotate 1080 degrees each second

            tail_rot_node.euler_rotation.x = tail_rot_node.euler_rotation.x % 360.0;
            tail_rot_node.euler_rotation.x += delta_time * 1080.0;

            scene_graph.update_node_transformations(&glm::identity());

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
                            camera.turn(mouse_input, delta_time);
                        }
                    }
                }
            });

            // Handle keyboard input
            pressed_keys.iter().for_each(|key| {
                match key {
                    VirtualKeyCode::W => camera.move_in_dir(VecDir::Forward, delta_time),
                    VirtualKeyCode::S => camera.move_in_dir(VecDir::Backward, delta_time),
                    VirtualKeyCode::A => camera.move_in_dir(VecDir::Left, delta_time),
                    VirtualKeyCode::D => camera.move_in_dir(VecDir::Right, delta_time),
                    VirtualKeyCode::R => disable_turn = !disable_turn,
                    VirtualKeyCode::Space => camera.move_in_dir(VecDir::Up, delta_time),
                    VirtualKeyCode::LControl => camera.move_in_dir(VecDir::Down, delta_time),
                    _ => { }
                }
            });

            unsafe {
                gl::ClearColor(0.05, 0.05, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
                
                scene_graph.draw(&camera);
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

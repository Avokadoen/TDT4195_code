use crate::gl_utils::shaders::program::Program;
use glm;

pub struct Camera {
    projection: glm::Mat4x4,
    transform: glm::Mat4x4,
    forward_dir: glm::Vec3,
    move_speed: f32
}

// impl Camera {
    
// }

pub struct CameraBuilder {
    projection: Option<glm::Mat4x4>,
    transform: Option<glm::Mat4x4>,
    forward_dir: Option<glm::Vec3>,
    move_speed: Option<f32>
}

impl CameraBuilder {
    pub fn init() -> Self {
        Self {
            projection: None,
            transform: None,
            forward_dir: None,
            move_speed: None
        }
    }

    pub fn projection(mut self, aspect: f32, fovy: f32, near: f32, far: f32) -> Self {
        self.projection = Some(glm::perspective::<f32>(
            aspect, 
            fovy, 
            near, 
            far
        ));

        self
    }

    pub fn transform(mut self, start_pos: &glm::Vec3) -> Self {
        self.transform = Some(glm::translate(&glm::identity::<f32, glm::U4>(), &start_pos));
        
        self
    }

    pub fn forward_dir(mut self, forward_dir: glm::Vec3) -> Self {
        self.forward_dir = Some(forward_dir);

        self
    }

    pub fn move_speed(mut self, move_speed: f32) -> Self {
        self.move_speed = Some(move_speed);

        self
    }

    // TODO: Return Result<Camera, CustomError> 
    pub fn build_and_attach_to_program(self, program: &mut Program) -> Camera {
        let projection = self.projection.expect("CameraBuiler has no projection");
        let transform = self.transform.expect("CameraBuiler has no transform");
        let forward_dir: glm::Vec3 = self.forward_dir.unwrap_or_else(|| {
            println!("Default forward_dir for CameraBuilder not supplied, using default");
            glm::vec3(0.0, 0.0, 1.0)
        });
        let move_speed: f32 = self.move_speed.unwrap_or_else(|| {
            println!("Default move_speed for CameraBuilder not supplied, using default");
            1.0
        });

        let camera = Camera {
            projection,
            transform,
            forward_dir,
            move_speed
        };

        if let Err(e) = program.locate_uniform("c_trans") {
            eprint!("Failed to find c_trans, probably loading wrong shader. err: {}", e);
        };

        if let Err(e) = program.set_uniform_matrix("c_trans", camera.transform.as_ptr(), gl::UniformMatrix4fv) {
            eprintln!("{}", e);
        };

        if let Err(e) = program.locate_uniform("projection") {
            eprint!("Failed to find projection, probably loading wrong shader. err: {}", e);
        };

        if let Err(e) = program.set_uniform_matrix("projection", camera.projection.as_ptr(), gl::UniformMatrix4fv) {
            eprintln!("{}", e);
        };

        return camera;
    }
}
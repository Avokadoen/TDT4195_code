use crate::gl_utils::{geometric_object::GeometricObject, mesh::Helicopter, scene_graph::Node, scene_graph::SceneNode, toolbox::Heading, toolbox::simple_heading_animation};

pub struct HelicopterNode {
    pub root_node: Node,
    pub body_node: Node,
    pub main_rotor_node: Node,
    pub tail_rotor_node: Node,
    pub door_node: Node,
    pub heading: Heading,
    pub heading_offset: f32,
    pub pos_offset: glm::Vec3,
}

impl HelicopterNode {
    const PI: f32 = 3.14159265359;
    const TWO_PI: f32 = 2.0 * HelicopterNode::PI;

    pub fn update(&mut self, delta_time: f32, elapsed: f32) {
        self.main_rotor_node.rotation.y = self.main_rotor_node.rotation.y % HelicopterNode::TWO_PI; // Avoid floating point errors
        self.main_rotor_node.rotation.y += delta_time * HelicopterNode::TWO_PI * 3.0; // 3 times each second

        self.tail_rotor_node.rotation.x = self.tail_rotor_node.rotation.x % HelicopterNode::TWO_PI;
        self.tail_rotor_node.rotation.x += delta_time * HelicopterNode::TWO_PI * 3.0;

        self.heading.update(elapsed + self.heading_offset);
        self.root_node.position.x = self.heading.x + self.pos_offset.x;
        self.root_node.position.z = self.pos_offset.y;
        self.root_node.position.z = self.heading.z + self.pos_offset.z;
        self.body_node.rotation.x = self.heading.pitch;
        self.body_node.rotation.y = self.heading.yaw;
        self.body_node.rotation.z = self.heading.roll;
    }
}

pub struct MyHelicopter {
    body_geometry: GeometricObject,
    main_rotor_geometry: GeometricObject,
    tail_rotor_geometry: GeometricObject,
    door_geometry: GeometricObject,
    max_instance: usize,
    last_instance: usize
}

impl MyHelicopter {
    pub fn init(program_id: u32, count: usize) -> Self {
        let transforms: Vec<glm::Mat4> = vec![glm::identity(); count];
        let (body_geometry, main_rotor_geometry, tail_rotor_geometry, door_geometry) = {
            let h = Helicopter::load("assets/objs/helicopter.obj");
            // We dissect helicopter to make it easier to take ownership of each mesh
            (
                h.body.into_geomtric_object(program_id, &transforms), 
                h.main_rotor.into_geomtric_object(program_id, &transforms), 
                h.tail_rotor.into_geomtric_object(program_id, &transforms), 
                h.door.into_geomtric_object(program_id, &transforms)
            )
        };

        Self {
            body_geometry,
            main_rotor_geometry,
            tail_rotor_geometry,
            door_geometry,
            max_instance: count,
            last_instance: 0
        }
    }

    // TODO: error not option
    pub fn create_helicopter_node(&mut self, heading_offset: f32, pos_offset: glm::Vec3) -> Option<HelicopterNode> {
        if self.last_instance >= self.max_instance {
            return None;
        }

        let mut root_node = SceneNode::new();
        let body_instance = self.body_geometry.create_geometric_instance(self.last_instance).expect("failed to create body instance");
        let mut body_node = SceneNode::from_vao(body_instance);
        root_node.add_child(&body_node);
            
        let main_rotor_instance = self.main_rotor_geometry.create_geometric_instance(self.last_instance).expect("failed to create main rotor instance");
        let mut main_rotor_node = SceneNode::from_vao(main_rotor_instance);
        body_node.add_child(&main_rotor_node);

        let tail_rot_instance = self.tail_rotor_geometry.create_geometric_instance(self.last_instance).expect("failed to create tail rotor instance");
        let mut tail_rotor_node = SceneNode::from_vao(tail_rot_instance);
        tail_rotor_node.set_reference_point(glm::vec3(0.35,2.3,10.4));
        body_node.add_child(&tail_rotor_node);
        
        let door_instance = self.door_geometry.create_geometric_instance(self.last_instance).expect("failed to create door instance");
        let door_node = SceneNode::from_vao(door_instance);
        body_node.add_child(&door_node);

        self.last_instance += 1;

        Some(HelicopterNode {
            root_node,
            body_node,
            main_rotor_node,
            tail_rotor_node,
            door_node,
            heading: Heading::new(),
            heading_offset,
            pos_offset
        })
    }
}
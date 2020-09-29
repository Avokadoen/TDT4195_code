use gl;
use gl::types::{GLuint, GLsizei, GLintptr};

use super::{bindable::Bindable, helpers, shaders::program::Program, vertex_attributes::VerticesAttributesPair};

pub struct GeometricObject {
    id: GLuint, // TODO: rename vao
    vbo_ids: Vec<GLuint>, // TODO: rename vbos
    pub instance_count: GLsizei,
    pub indices_count: GLsizei,
    pub buffer_count: GLsizei
}


impl Drop for GeometricObject {
    fn drop(&mut self) {
        unsafe {
            for buffer in &self.vbo_ids {
                // TODO: as we use vector, we can't delete all at once
                // Seems the only possible solutions is a experimental function called "leak"
                gl::DeleteBuffers(1, buffer); 
            }
            gl::DeleteVertexArrays(1, self.id as *const GLuint);
        }
    }
}

impl Bindable for GeometricObject {
    fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_ids[GeometricObject::INST_INDEX]);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vbo_ids[GeometricObject::ELEM_INDEX]);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }
}

// TODO: errors
impl GeometricObject {
    pub const ELEM_INDEX: usize = 0;
    pub const INST_INDEX: usize = 1;
    
    // TODO: this should read shader string and modify locations to fit with buffers
    pub fn init<T>(buffer_attrib_pairs: &Vec<VerticesAttributesPair<T>>, indices: &Vec<u32>, instance_transforms: &Vec<glm::Mat4>) -> Self  {
        let mut id: GLuint = 0;
        let buffer_count = buffer_attrib_pairs.len() + 2;
        let mut instance_location = 1; // location in shader
        let mut vbo_ids = Vec::<GLuint>::with_capacity(buffer_count);

        unsafe {
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);

            let mut elem_buf: gl::types::GLuint = 0;
            gl::GenBuffers(1, &mut elem_buf);
            vbo_ids.push(elem_buf);

            let mut instance_buf: gl::types::GLuint = 0;
            gl::GenBuffers(1, &mut instance_buf);
            vbo_ids.push(instance_buf);

            for vert_attrib_pair in buffer_attrib_pairs {
                // instantiate vertices buffer
                let mut buff: gl::types::GLuint = 0;
                gl::GenBuffers(1, &mut buff);
                gl::BindBuffer(gl::ARRAY_BUFFER, buff);
                vbo_ids.push(buff);

                gl::BufferData(
                    gl::ARRAY_BUFFER, 
                    helpers::byte_size_of_array(&vert_attrib_pair.buffer_data),
                    helpers::array_to_c_void(&vert_attrib_pair.buffer_data),
                    gl::STATIC_DRAW
                );

                // Vertex attributes
                let size_of_type = helpers::size_of::<T>();
                let total_components: gl::types::GLint = vert_attrib_pair.attributes.iter()
                    .map(|a| {
                        a.size
                    })
                    .sum();

                let stride = total_components * size_of_type;
                for attrib in &vert_attrib_pair.attributes {
                    if instance_location <= attrib.index {
                        instance_location = attrib.index + 1;
                    }

                    gl::EnableVertexAttribArray(attrib.index);
                    gl::VertexAttribPointer(
                        attrib.index,                           // index of the generic vertex attribute ("layout (location = 0)")
                        attrib.size,                            // the number of components per generic vertex attribute
                        vert_attrib_pair.data_type,             // data type
                        gl::FALSE,                              // normalized (int-to-float conversion)
                        stride,                                 // stride (byte offset between consecutive attributes)
                        helpers::offset::<T>(attrib.offset)     // offset of the first component
                    );
                }
            }
            
            // instantiate element buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_ids[GeometricObject::ELEM_INDEX]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER, 
                helpers::byte_size_of_array(indices),
                helpers::array_to_c_void(indices),
                gl::STATIC_DRAW
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ids[GeometricObject::INST_INDEX]);
            gl::BufferData(
                gl::ARRAY_BUFFER, 
                helpers::byte_size_of_array(&instance_transforms),
                helpers::array_to_c_void(&instance_transforms),
                gl::STATIC_DRAW
            );
            
            let mat_size = std::mem::size_of::<glm::Mat4>() as i32;
            let vec_size = std::mem::size_of::<glm::Vec4>() as u32;
            for i in 0..4 {
                let attrib_index = instance_location + i;
                gl::EnableVertexAttribArray(attrib_index);
                gl::VertexAttribPointer(
                    attrib_index,                           // currently shader expects location=1
                    4,                            
                    gl::FLOAT,            
                    gl::FALSE,                              
                    mat_size,                                
                    (i * vec_size) as *const usize as *const core::ffi::c_void
                );
                
                gl::VertexAttribDivisor(attrib_index, 1);
            }

            // Better safe than sorry :) 
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Self {
            id,
            vbo_ids,
            indices_count: indices.len() as GLsizei,
            instance_count: instance_transforms.len() as GLsizei,
            buffer_count: buffer_count as GLsizei
        }
    }

    pub fn draw_all(&self, program: &Program) {
        self.bind();

        unsafe {
            gl::UseProgram(program.program_id);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                self.indices_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
                self.instance_count
            ); 
            gl::UseProgram(0);
        }

        self.unbind();
    }

    pub fn update_transform(&self, index: i32, new_transform: &glm::Mat4) {
        if self.instance_count < index {
            return; // ERROR
        }

        let mat4_size = std::mem::size_of::<glm::Mat4>();
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_ids[Self::INST_INDEX]);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (mat4_size as i32 * index) as GLintptr,
                mat4_size as isize,
                new_transform.as_ptr() as *const f32 as *const core::ffi::c_void
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}
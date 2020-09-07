use gl::types;

// TODO: not really a pair
// TODO: having attributes with different T is not really supported in the same object ... 
pub struct VerticesAttributesPair<T> {
    pub buffer_data: Vec::<T>,
    pub attributes: Vec<VertexAttribute>,
    pub data_type: types::GLenum // TODO: not sure if type is always the same for all attributes in the same buffer
}

impl<T> VerticesAttributesPair<T> {
    pub fn init(buffer_data: Vec::<T>, data_type: types::GLenum) -> Self {
        Self {
            buffer_data,
            attributes: Vec::with_capacity(2),
            data_type
        }
    }
    
    #[must_use = "Need atleast one attribute"]
    pub fn add_attribute(
        mut self, 
        id: types::GLuint, 
        index: types::GLuint, 
        size: types::GLint,
        offset: u32
    ) -> Self {
        self.attributes.push(
            VertexAttribute::init(id, index, size, offset)
        );

        self
    }
}

pub struct VertexAttribute {
    pub id: types::GLuint, // TODO: id and index can probably be merged
    pub index: types::GLuint,
    pub size: types::GLint,
    pub offset: u32
}

impl VertexAttribute {
    // TODO: i'm sure this is unnecessary boilerplate, find how to remove this code. 
    fn init(
        id: types::GLuint,
        index: types::GLuint, 
        size: types::GLint,
        offset: u32
    ) -> Self {
        VertexAttribute {
            id,
            index,
            size,
            offset
        }
    }
}
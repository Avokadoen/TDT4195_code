#version 430 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normals;
layout (location = 2) in vec4 colors;
layout (location = 3) in mat4 instance_transform;

uniform mat4 camera;

out vec3 vert_normals;
out vec4 vert_colors;

void main()
{
    vert_normals = normals;
    vert_colors = colors;
    gl_Position = camera * instance_transform * vec4(position, 1.0);
}
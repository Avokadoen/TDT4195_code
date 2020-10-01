#version 430 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec4 color;
layout (location = 3) in mat4 instance_transform;

uniform mat4 camera;

out vec3 vert_normal;
out vec4 vert_color;

void main()
{
    vert_normal = normalize(mat3(instance_transform) * normal);
    vert_color = color;
    gl_Position = camera * instance_transform * vec4(position, 1.0);
}
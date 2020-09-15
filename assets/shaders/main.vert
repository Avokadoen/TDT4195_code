#version 430 core
layout (location = 0) in vec3 position;
layout (location = 1) in mat4 instance_transform;

uniform mat4 camera;

void main()
{
    gl_Position = camera * instance_transform * vec4(position, 1.0);
}
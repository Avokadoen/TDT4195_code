#version 430 core

in vec3 vert_normal;
in vec4 vert_color;

out vec4 color;

void main()
{
    // TODO: this should be a uniform
    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
    color = vert_color * max(dot(vert_normal, -lightDirection), 0);
    color.w = 1;
}
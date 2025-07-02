/* vim: set filetype=glsl : */

export const fragmentSource = `#version 300 es
precision mediump float;

in vec2 vTexCoord;
out vec4 outColor;

uniform sampler2D uTexture;

void main()
{
  float value = texture(uTexture, vTexCoord).r;
  outColor = vec4(vec3(value), 1.0);
}`;

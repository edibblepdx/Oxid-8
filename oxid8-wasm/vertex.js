/* vim: set filetype=glsl : */

export const vertexSource = `#version 300 es
precision mediump float;

layout (location = 0) in vec2 aPosition;
out vec2 vTexCoord;

void main()
{
  vTexCoord = (aPosition + 1.0) * 0.5; // Convert from NDC to [0,1]
  gl_Position = vec4(aPosition, 0.0, 1.0);
}`;

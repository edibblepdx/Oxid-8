// @ts-check

/**
 * @file webgl-utils.js
 * @author ethan dibble
 * @date 2024-12-30
 * @description WebGL utility functions.
 */

/**
 * Creates and compiles a new shader.
 *
 * @param {!WebGL2RenderingContext} gl
 * @param {!number} type
 * @param {!string} source
 * @returns {?WebGLShader}
 */
export function createShader (gl, type, source) {
    const shader = gl.createShader(type);
    if (!shader) return null;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error("ERROR::SHADER::COMPILATION_FAILED\n" 
            + gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
    }
    return shader;
}

/**
 * Creates, links, and validates a new shader program.
 *
 * @param {!WebGL2RenderingContext} gl
 * @param {!number} vertexShader
 * @param {!number} fragmentShader
 * @returns {?WebGLShader}
 */
export function createProgram (gl, vertexShader, fragmentShader) {
    const shaderProgram = gl.createProgram();
    if (!shaderProgram) return null;
    gl.attachShader(shaderProgram, vertexShader);
    gl.attachShader(shaderProgram, fragmentShader);
    // link shader program
    gl.linkProgram(shaderProgram);
    if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
        console.error("ERROR::SHADER::PROGRAM::LINKING_FAILED\n"
            + gl.getProgramInfoLog(shaderProgram));
        gl.deleteProgram(shaderProgram);
        return null;
    }
    // validate shader program
    gl.validateProgram(shaderProgram);
    if (!gl.getProgramParameter(shaderProgram, gl.VALIDATE_STATUS)) {
        console.error("ERROR::SHADER::PROGRAM::VALIDATION_FAILED\n"
            + gl.getProgramInfoLog(shaderProgram));
        gl.deleteProgram(shaderProgram);
        return null;
    }
    return shaderProgram;
}

/**
 * Creates a new texture object.
 * 
 * @param {!WebGL2RenderingContext} gl 
 * @param {!string} src 
 * @returns {?WebGLTexture}
 */
export function createTexture (gl, src) {
    if (!src) {
        console.error("ERROR::TEXTURE::INVALID_SRC");
        return null;
    }
    const texture = gl.createTexture();
    const image = new Image(); 
    image.onload = () => {
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        console.log("texture loaded");
    }
    image.src = src;
    return texture;
}

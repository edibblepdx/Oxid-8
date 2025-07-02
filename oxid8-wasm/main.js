import init, { Emu, Framebuffer } from "./pkg/oxid8_wasm.js";
import { vertexSource } from "./vertex.js";
import { fragmentSource } from "./fragment.js";
import { createShader, createProgram } from "./webgl-utils.js";

init().then((wasm) => {
  class App {
    constructor() {
      this.cpuInterval = 1000 / 700; // 700Hz
      this.timerInterval = 1000 / 60; // 60Hz

      // Create interpreter core and load font
      this.core = new Emu();
      this.core.load_font();

      // Get frame buffer
      this.buffer = new Uint8Array(
        wasm.memory.buffer,
        this.core.frame.as_ptr(),
        this.core.frame.area,
      );

      // Timers
      this.cpuTime = 0;
      this.timerTime = 0;
      this.previousRAF_ = null;
    }

    initialize() {
      this.canvas = document.getElementById('canvas');
      this.gl = this.canvas.getContext('webgl2');

      //this.canvas.width = this.canvas.clientWidth * window.devicePixelRatio;
      //this.canvas.height = this.canvas.clientHeight * window.devicePixelRatio;
      //this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);

      if (!this.gl) {
        console.error("ERROR::WEBGL2::INITIALIZATION_ERROR");
        return;
      }

      this.gl.clearColor(0.0, 0.0, 0.0, 1.0);
      this.gl.clear(this.gl.COLOR_BUFFER_BIT);

      window.addEventListener('resize', () => {
        this.onWindowResize_();
      }, false);

      this.setupProgram_();
      this.raf_();
    }

    setupProgram_() {
      this.texture = this.gl.createTexture();
      this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);

      this.gl.texImage2D(
        this.gl.TEXTURE_2D, 0, this.gl.LUMINANCE,
        Framebuffer.width, Framebuffer.height, 0,
        this.gl.LUMINANCE, this.gl.UNSIGNED_BYTE, 
        this.buffer
      );
      // flip texture
      // y-flip is deprecated so find another method
      this.gl.pixelStorei(this.gl.UNPACK_FLIP_Y_WEBGL, true);
      this.gl.generateMipmap(this.gl.TEXTURE_2D);

      this.gl.texParameteri(
        this.gl.TEXTURE_2D,
        this.gl.TEXTURE_WRAP_S,
        this.gl.CLAMP_TO_EDGE
      );
      this.gl.texParameteri(
        this.gl.TEXTURE_2D,
        this.gl.TEXTURE_WRAP_T,
        this.gl.CLAMP_TO_EDGE
      );
      this.gl.texParameteri(
        this.gl.TEXTURE_2D,
        this.gl.TEXTURE_MIN_FILTER,
        this.gl.LINEAR
      );

      this.gl.bindTexture(this.gl.TEXTURE_2D, null);

      // full screen quad
      const vertices = new Float32Array([
        -1, -1, 1, -1, -1, 1, // Triangle 1
        -1, 1, 1, -1, 1, 1, // Triangle 2
      ]);

      this.vao = this.gl.createVertexArray();
      this.vbo = this.gl.createBuffer();

      // bind vao
      this.gl.bindVertexArray(this.vao);

      this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.vbo);
      this.gl.bufferData(this.gl.ARRAY_BUFFER, vertices, this.gl.STATIC_DRAW);

      this.gl.vertexAttribPointer(0, 2, this.gl.FLOAT, false, 0, 0);
      this.gl.enableVertexAttribArray(0);

      // unbind vao
      this.gl.bindVertexArray(null);

      // build and link shader program
      const vs = createShader(this.gl, this.gl.VERTEX_SHADER, vertexSource);
      const fs = createShader(this.gl, this.gl.FRAGMENT_SHADER, fragmentSource);
      this.shaderProgram = createProgram(this.gl, vs, fs);

      // texture uniform location
      this.uTexLoc = this.gl.getUniformLocation(
        this.shaderProgram,
        "uTexture"
      );
    }

    onWindowResize_() {
      //this.canvas.width = this.canvas.clientWidth * devicePixelRatio;
      //this.canvas.height = this.canvas.clientHeight * devicePixelRatio;
      //
      //this.canvas.width = this.canvas.clientWidth * window.devicePixelRatio;
      //this.canvas.height = this.canvas.clientHeight * window.devicePixelRatio;
      //this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    }

    step_(deltaTime) {
      this.cpuTime += deltaTime;
      this.timerTime += deltaTime;

      while (this.cpuTime >= this.cpuInterval) {
        this.core.run_cycle();
        this.cpuTime -= this.cpuInterval;
      }

      let redraw = false;
      while (this.timerTime >= this.cpuInterval) {
        redraw = true;
        this.core.dec_timers();
        this.timerTime -= this.timerInterval;
      }

      if (redraw) {
        // write to buffer
        this.core.draw();
        this.buffer = new Uint8Array(
          wasm.memory.buffer,
          this.core.frame.as_ptr(),
          this.core.frame.area,
        );

        // update texture from WASM memory
        this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
        this.gl.texImage2D(
          this.gl.TEXTURE_2D, 0, this.gl.LUMINANCE,
          Framebuffer.width, Framebuffer.height, 0,
          this.gl.LUMINANCE, this.gl.UNSIGNED_BYTE, 
          this.buffer
        );
        this.gl.bindTexture(this.gl.TEXTURE_2D, null);
      }
    }

    loadROM_(rom_data) {
      this.core.load_rom_as_bytes(rom_data);
      console.log(rom_data);
    }

    raf_() {
      requestAnimationFrame((t) => {
        if (this.previousRAF_ === null) {
          this.previousRAF_ = t;
        }
        this.step_(t - this.previousRAF_);

        // clear screen and bind shader program
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
        this.gl.useProgram(this.shaderProgram);

        // bind texture
        this.gl.activeTexture(this.gl.TEXTURE0);
        this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
        this.gl.uniform1i(this.uTexLoc, 0);

        // bind vao and draw
        this.gl.bindVertexArray(this.vao);
        this.gl.drawArrays(this.gl.TRIANGLES, 0, 6);

        // unbind vao and shader program
        this.gl.bindVertexArray(null);
        this.gl.useProgram(null);

        this.previousRAF_ = t;
        this.raf_();
      });
    }
  }


  /*
  let APP_ = null;

  window.addEventListener('DOMContentLoaded', () => {
    APP_ = new App();
    APP_.initialize();
  });
  */

  let APP_ = new App();

  // Add ROM file loader
  document
    .getElementById("romInput")
    .addEventListener("change",
      async (event) => {

    const file = event.target.files[0];
    if (!file) return;

    const buffer = await file.arrayBuffer();
    const romData = new Uint8Array(buffer);
    APP_.loadROM_(romData);
    APP_.initialize();
  });
});

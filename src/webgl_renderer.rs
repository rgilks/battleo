use crate::agent::Agent;
use crate::resource::Resource;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader,
    WebGlUniformLocation,
};

pub struct WebGlRenderer {
    gl: WebGlRenderingContext,
    program: WebGlProgram,
    agent_buffer: WebGlBuffer,
    resource_buffer: WebGlBuffer,
    agent_count: u32,
    resource_count: u32,
    canvas_size_location: Option<WebGlUniformLocation>,
}

impl WebGlRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let gl = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        // Set up viewport
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        gl.viewport(0, 0, width, height);

        // Enable blending for transparency
        gl.enable(WebGlRenderingContext::BLEND);
        gl.blend_func(
            WebGlRenderingContext::SRC_ALPHA,
            WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        // Create shader program
        let program = Self::create_shader_program(&gl)?;

        // Create buffers
        let agent_buffer = gl.create_buffer().ok_or("Failed to create agent buffer")?;
        let resource_buffer = gl
            .create_buffer()
            .ok_or("Failed to create resource buffer")?;

        // Get uniform locations
        let canvas_size_location = gl.get_uniform_location(&program, "u_canvas_size");

        Ok(WebGlRenderer {
            gl,
            program,
            agent_buffer,
            resource_buffer,
            agent_count: 0,
            resource_count: 0,
            canvas_size_location,
        })
    }

    fn create_shader_program(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"precision mediump float;
attribute vec2 a_position;
attribute vec3 a_color;
uniform vec2 u_canvas_size;
varying vec3 v_color;
void main() {
    // Transform from pixel coordinates to normalized device coordinates
    vec2 ndc = (a_position / u_canvas_size) * 2.0 - 1.0;
    ndc.y = -ndc.y; // Flip Y axis
    gl_Position = vec4(ndc, 0.0, 1.0);
    gl_PointSize = 6.0;
    v_color = a_color;
}"#,
        )?;

        let fragment_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"precision mediump float;
varying vec3 v_color;
void main() {
    gl_FragColor = vec4(v_color, 1.0);
}"#,
        )?;

        let program = gl.create_program().ok_or("Failed to create program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        let link_status = gl.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS);
        if link_status.as_bool().unwrap_or(false) {
            Ok(program)
        } else {
            let error = gl.get_program_info_log(&program).unwrap_or_default();
            Err(format!("Failed to link shader program: {}", error).into())
        }
    }

    fn create_shader(
        gl: &WebGlRenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, JsValue> {
        let shader_type_str = if shader_type == WebGlRenderingContext::VERTEX_SHADER {
            "vertex"
        } else {
            "fragment"
        };

        let shader = gl
            .create_shader(shader_type)
            .ok_or("Failed to create shader")?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        let compile_status =
            gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS);
        if compile_status.as_bool().unwrap_or(false) {
            Ok(shader)
        } else {
            let error = gl.get_shader_info_log(&shader).unwrap_or_default();
            let shader_type_str = if shader_type == WebGlRenderingContext::VERTEX_SHADER {
                "vertex"
            } else {
                "fragment"
            };
            Err(format!("Failed to compile {} shader: {}", shader_type_str, error).into())
        }
    }

    pub fn update_agents(&mut self, agents: &[Agent]) {
        self.agent_count = agents.len() as u32;

        // Convert agents to GPU data with colors
        let mut agent_data = Vec::new();
        for agent in agents {
            // Position (vec2)
            agent_data.extend_from_slice(&(agent.x as f32).to_le_bytes());
            agent_data.extend_from_slice(&(agent.y as f32).to_le_bytes());

            // Color (vec3) based on genes
            let hue = (agent.genes.speed * 100.0 + agent.genes.sense_range * 50.0) % 360.0;
            let saturation = 0.7 + agent.genes.size * 0.2;
            let lightness = 0.5 + agent.energy * 0.002;

            // Convert HSL to RGB
            let (r, g, b) = Self::hsl_to_rgb(hue as f32, saturation as f32, lightness as f32);
            agent_data.extend_from_slice(&(r as f32).to_le_bytes());
            agent_data.extend_from_slice(&(g as f32).to_le_bytes());
            agent_data.extend_from_slice(&(b as f32).to_le_bytes());
        }

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.agent_buffer),
        );
        self.gl.buffer_data_with_u8_array(
            WebGlRenderingContext::ARRAY_BUFFER,
            &agent_data,
            WebGlRenderingContext::DYNAMIC_DRAW,
        );
    }

    pub fn update_resources(&mut self, resources: &[Resource]) {
        self.resource_count = resources.len() as u32;

        // Convert resources to GPU data with colors
        let mut resource_data = Vec::new();
        for resource in resources {
            // Position (vec2)
            resource_data.extend_from_slice(&(resource.x as f32).to_le_bytes());
            resource_data.extend_from_slice(&(resource.y as f32).to_le_bytes());

            // Color (vec3) based on energy - green to yellow to orange
            let energy_ratio = (resource.energy / 100.0).min(1.0);
            let r = energy_ratio;
            let g = 1.0 - energy_ratio * 0.5;
            let b = 0.0;

            resource_data.extend_from_slice(&(r as f32).to_le_bytes());
            resource_data.extend_from_slice(&(g as f32).to_le_bytes());
            resource_data.extend_from_slice(&(b as f32).to_le_bytes());
        }

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.resource_buffer),
        );
        self.gl.buffer_data_with_u8_array(
            WebGlRenderingContext::ARRAY_BUFFER,
            &resource_data,
            WebGlRenderingContext::DYNAMIC_DRAW,
        );
    }

    pub fn render(&self) {
        // Clear the canvas
        self.gl.clear_color(0.1, 0.1, 0.18, 1.0);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        // Use our shader program
        self.gl.use_program(Some(&self.program));

        // Set canvas size uniform
        if let Some(ref location) = self.canvas_size_location {
            self.gl.uniform2f(Some(location), 800.0, 600.0);
        }

        // Render agents
        self.render_entities(&self.agent_buffer, self.agent_count);

        // Render resources
        self.render_entities(&self.resource_buffer, self.resource_count);
    }

    fn render_entities(&self, buffer: &WebGlBuffer, count: u32) {
        if count == 0 {
            return;
        }

        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));

        // Position attribute (vec2)
        let position_location = self.gl.get_attrib_location(&self.program, "a_position") as u32;
        self.gl.enable_vertex_attrib_array(position_location);
        self.gl.vertex_attrib_pointer_with_i32(
            position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            20, // 2 floats for position + 3 floats for color
            0,
        );

        // Color attribute (vec3)
        let color_location = self.gl.get_attrib_location(&self.program, "a_color") as u32;
        self.gl.enable_vertex_attrib_array(color_location);
        self.gl.vertex_attrib_pointer_with_i32(
            color_location,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            20, // 2 floats for position + 3 floats for color
            8,  // Offset to color data
        );

        // Draw points
        self.gl
            .draw_arrays(WebGlRenderingContext::POINTS, 0, count as i32);
    }

    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        (r + m, g + m, b + m)
    }
}

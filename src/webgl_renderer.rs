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
        web_sys::console::log_1(&"Creating WebGL context...".into());
        let gl = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;
        web_sys::console::log_1(&"WebGL context created successfully!".into());

        // Set up viewport
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        gl.viewport(0, 0, width, height);
        web_sys::console::log_1(&format!("Set viewport: {}x{}", width, height).into());

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
            r#"attribute vec2 a_position;
uniform vec2 u_canvas_size;
void main() {
    // Transform from pixel coordinates to normalized device coordinates
    // Pixel coordinates: (0,0) at top-left, (width,height) at bottom-right
    // NDC coordinates: (-1,-1) at bottom-left, (1,1) at top-right
    vec2 ndc = (a_position / u_canvas_size) * 2.0 - 1.0;
    ndc.y = -ndc.y; // Flip Y axis
    gl_Position = vec4(ndc, 0.0, 1.0);
    gl_PointSize = 5.0;
}"#,
        )?;

        let fragment_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"void main() {
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}"#,
        )?;

        let program = gl.create_program().ok_or("Failed to create program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        let link_status = gl.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS);
        web_sys::console::log_1(&format!("Link status: {:?}", link_status).into());

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
        web_sys::console::log_1(&format!("Creating {} shader...", shader_type_str).into());

        let shader = gl
            .create_shader(shader_type)
            .ok_or("Failed to create shader")?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        web_sys::console::log_1(&format!("Compiled {} shader", shader_type_str).into());

        let compile_status =
            gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS);
        web_sys::console::log_1(
            &format!(
                "Compile status for {} shader: {:?}",
                shader_type_str, compile_status
            )
            .into(),
        );

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
        web_sys::console::log_1(&format!("Updating {} agents", self.agent_count).into());

        // Convert agents to GPU data
        let mut agent_data = Vec::new();
        for agent in agents {
            // Position (vec2) only
            agent_data.extend_from_slice(&(agent.x as f32).to_le_bytes());
            agent_data.extend_from_slice(&(agent.y as f32).to_le_bytes());
        }

        web_sys::console::log_1(&format!("Agent buffer size: {} bytes", agent_data.len()).into());

        // Log first few agent positions for debugging
        if !agents.is_empty() {
            web_sys::console::log_1(
                &format!("First agent position: ({}, {})", agents[0].x, agents[0].y).into(),
            );
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
        web_sys::console::log_1(&format!("Updating {} resources", self.resource_count).into());

        // Convert resources to GPU data
        let mut resource_data = Vec::new();
        for resource in resources {
            // Position (vec2) only
            resource_data.extend_from_slice(&(resource.x as f32).to_le_bytes());
            resource_data.extend_from_slice(&(resource.y as f32).to_le_bytes());
        }

        web_sys::console::log_1(
            &format!("Resource buffer size: {} bytes", resource_data.len()).into(),
        );

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

        web_sys::console::log_1(&format!("Rendering {} entities", count).into());

        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));

        // Position attribute only
        let position_location = self.gl.get_attrib_location(&self.program, "a_position") as u32;
        self.gl.enable_vertex_attrib_array(position_location);
        self.gl.vertex_attrib_pointer_with_i32(
            position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            8,
            0,
        );

        // Draw points
        self.gl
            .draw_arrays(WebGlRenderingContext::POINTS, 0, count as i32);
    }
}

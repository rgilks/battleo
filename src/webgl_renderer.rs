use crate::agent::{Agent, DeathReason};
use crate::resource::Resource;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader,
    WebGlUniformLocation,
};

pub struct WebGlRenderer {
    gl: WebGlRenderingContext,
    agent_program: WebGlProgram,
    resource_program: WebGlProgram,
    trail_program: WebGlProgram,
    agent_buffer: WebGlBuffer,
    resource_buffer: WebGlBuffer,
    trail_buffer: WebGlBuffer,
    agent_count: u32,
    resource_count: u32,
    trail_count: u32,
    canvas_size_location: Option<WebGlUniformLocation>,
    time_location: Option<WebGlUniformLocation>,
    agent_positions: Vec<(f32, f32)>,
    resource_positions: Vec<(f32, f32)>,
    resource_growth_states: Vec<f32>, // Track growth state for each resource
    time: f32,
    canvas_width: u32,
    canvas_height: u32,
}

impl WebGlRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        web_sys::console::log_1(&"Attempting to create WebGL context...".into());

        let gl = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        web_sys::console::log_1(&"WebGL context created successfully!".into());

        // Set up viewport
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        gl.viewport(0, 0, width, height);

        // Enable blending for transparency and glow effects
        gl.enable(WebGlRenderingContext::BLEND);
        gl.blend_func(
            WebGlRenderingContext::SRC_ALPHA,
            WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        // Enable depth testing for 3D-like effects
        gl.enable(WebGlRenderingContext::DEPTH_TEST);
        gl.depth_func(WebGlRenderingContext::LEQUAL);

        // Create shader programs
        web_sys::console::log_1(&"Creating agent shader program...".into());
        let agent_program = Self::create_agent_shader_program(&gl)?;
        web_sys::console::log_1(&"Creating resource shader program...".into());
        let resource_program = Self::create_resource_shader_program(&gl)?;
        web_sys::console::log_1(&"Creating trail shader program...".into());
        let trail_program = Self::create_trail_shader_program(&gl)?;
        web_sys::console::log_1(&"All shader programs created successfully!".into());

        // Create buffers
        let agent_buffer = gl.create_buffer().ok_or("Failed to create agent buffer")?;
        let resource_buffer = gl
            .create_buffer()
            .ok_or("Failed to create resource buffer")?;
        let trail_buffer = gl.create_buffer().ok_or("Failed to create trail buffer")?;

        // Get uniform locations
        let canvas_size_location = gl.get_uniform_location(&agent_program, "u_canvas_size");
        let time_location = gl.get_uniform_location(&agent_program, "u_time");

        Ok(WebGlRenderer {
            gl,
            agent_program,
            resource_program,
            trail_program,
            agent_buffer,
            resource_buffer,
            trail_buffer,
            agent_count: 0,
            resource_count: 0,
            trail_count: 0,
            canvas_size_location,
            time_location,
            agent_positions: Vec::new(),
            resource_positions: Vec::new(),
            resource_growth_states: Vec::new(),
            time: 0.0,
            canvas_width: width as u32,
            canvas_height: height as u32,
        })
    }

    fn create_agent_shader_program(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"precision highp float;
attribute vec2 a_position;
attribute vec3 a_color;
attribute float a_size;
attribute float a_energy;
uniform vec2 u_canvas_size;
uniform float u_time;
varying vec3 v_color;
varying float v_energy;
varying vec2 v_position;
varying float v_size;

void main() {
    // Transform from pixel coordinates to normalized device coordinates
    vec2 ndc = (a_position / u_canvas_size) * 2.0 - 1.0;
    ndc.y = -ndc.y; // Flip Y axis
    
    // Add subtle movement based on time and position
    float movement = sin(u_time * 2.0 + a_position.x * 0.01) * 0.0001;
    ndc += vec2(movement, movement * 0.5);
    
    gl_Position = vec4(ndc, 0.0, 1.0);
    
            // Dynamic point size based on energy and time
        float base_size = a_size * 30.0; // Much larger base size for visibility
        float pulse = sin(u_time * 3.0 + a_position.x * 0.1) * 0.2 + 1.0;
        float energy_scale = 0.5 + a_energy * 0.01;
        gl_PointSize = base_size * pulse * energy_scale;
    
    v_color = a_color;
    v_energy = a_energy;
    v_position = a_position;
    v_size = a_size;
}"#,
        )?;

        let fragment_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"precision highp float;
varying vec3 v_color;
varying float v_energy;
varying vec2 v_position;
varying float v_size;
uniform float u_time;

void main() {
    vec2 center = gl_PointCoord - 0.5;
    float dist = length(center);
    
    // Create a soft circular particle with enhanced edges
    float alpha = 1.0 - smoothstep(0.0, 0.5, dist);
    
    // Add inner core with bright center
    float core = 1.0 - smoothstep(0.0, 0.2, dist);
    vec3 core_color = mix(v_color, vec3(1.0), 0.6);
    
    // Add inner glow
    float inner_glow = 1.0 - smoothstep(0.0, 0.4, dist);
    vec3 glow_color = mix(v_color, vec3(1.0), 0.4);
    
    // Add outer glow based on energy with rainbow effect
    float outer_glow = 1.0 - smoothstep(0.4, 1.0, dist);
    float energy_glow = v_energy * 0.015;
    
    // Create rainbow outer glow
    float rainbow_hue = mod(u_time * 0.5 + v_position.x * 0.01, 1.0);
    vec3 rainbow_color = vec3(
        sin(rainbow_hue * 6.28) * 0.5 + 0.5,
        sin((rainbow_hue + 0.33) * 6.28) * 0.5 + 0.5,
        sin((rainbow_hue + 0.66) * 6.28) * 0.5 + 0.5
    );
    vec3 outer_glow_color = mix(v_color, rainbow_color, 0.7);
    
    // Combine all effects
    vec3 final_color = mix(v_color, core_color, core * 0.8);
    final_color = mix(final_color, glow_color, inner_glow * 0.6);
    final_color = mix(final_color, outer_glow_color, outer_glow * energy_glow);
    
    // Add dynamic pulsing effect
    float pulse = sin(u_time * 3.0 + v_position.x * 0.1) * 0.15 + 0.85;
    final_color *= pulse;
    
    // Add enhanced sparkle effect
    float sparkle1 = sin(u_time * 10.0 + v_position.x * 0.8) * sin(u_time * 7.0 + v_position.y * 0.6);
    float sparkle2 = sin(u_time * 6.0 + v_position.x * 0.4) * cos(u_time * 8.0 + v_position.y * 0.9);
    float sparkle = max(0.0, (sparkle1 + sparkle2) * 0.4);
    final_color += vec3(sparkle * 0.8);
    
    // Add energy-based color shift
    float energy_shift = sin(u_time * 2.0 + v_energy * 0.1) * 0.1;
    final_color += vec3(energy_shift, energy_shift * 0.5, energy_shift * 0.2);
    
    // Add subtle rotation effect
    float angle = atan(center.y, center.x);
    float rotation = sin(angle * 4.0 + u_time * 2.0) * 0.05 + 0.95;
    final_color *= rotation;
    
    gl_FragColor = vec4(final_color, alpha);
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
            Err(format!("Failed to link agent shader program: {}", error).into())
        }
    }

    fn create_resource_shader_program(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"precision highp float;
attribute vec2 a_position;
attribute vec3 a_color;
attribute float a_energy;
attribute float a_growth;
uniform vec2 u_canvas_size;
uniform float u_time;
varying vec3 v_color;
varying float v_energy;
varying float v_growth;
varying vec2 v_position;

void main() {
    vec2 ndc = (a_position / u_canvas_size) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    
    // Add gentle floating motion
    float float_offset = sin(u_time * 1.5 + a_position.x * 0.02) * 0.0002;
    ndc.y += float_offset;
    
    gl_Position = vec4(ndc, 0.0, 1.0);
    
    // Size based on energy and growth state
    float base_size = 20.0 + a_energy * 0.5;
    float growth_size = base_size * a_growth; // Scale by growth state
    gl_PointSize = growth_size;
    
    v_color = a_color;
    v_energy = a_energy;
    v_growth = a_growth;
    v_position = a_position;
}"#,
        )?;

        let fragment_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"precision highp float;
varying vec3 v_color;
varying float v_energy;
varying float v_growth;
varying vec2 v_position;
uniform float u_time;

void main() {
    vec2 center = gl_PointCoord - 0.5;
    float dist = length(center);
    
    // Create a glowing resource particle with enhanced effects
    float alpha = 1.0 - smoothstep(0.0, 0.6, dist);
    
    // Bright inner core
    float core = 1.0 - smoothstep(0.0, 0.15, dist);
    vec3 core_color = mix(v_color, vec3(1.0), 0.8);
    
    // Middle glow layer
    float middle_glow = 1.0 - smoothstep(0.15, 0.4, dist);
    vec3 middle_color = mix(v_color, vec3(1.0, 1.0, 0.9), 0.5);
    
    // Outer glow with energy-based intensity
    float outer_glow = 1.0 - smoothstep(0.4, 1.2, dist);
    float energy_intensity = v_energy * 0.02;
    vec3 outer_color = mix(v_color, vec3(1.0, 0.8, 0.4), 0.6);
    
    // Combine all layers
    vec3 final_color = mix(v_color, core_color, core * 0.9);
    final_color = mix(final_color, middle_color, middle_glow * 0.7);
    final_color = mix(final_color, outer_color, outer_glow * energy_intensity);
    
    // Add dynamic energy-based pulsing
    float energy_pulse = sin(u_time * 1.5 + v_energy * 0.05) * 0.25 + 0.75;
    final_color *= energy_pulse;
    
    // Add floating motion effect
    float float_effect = sin(u_time * 2.0 + v_position.x * 0.02) * 0.1 + 0.9;
    final_color *= float_effect;
    
    // Add enhanced rotation effect
    float angle = atan(center.y, center.x);
    float rotation = sin(angle * 5.0 + u_time * 1.5) * 0.15 + 0.85;
    final_color *= rotation;
    
    // Add energy-based color enhancement
    float energy_enhance = v_energy * 0.01;
    final_color += vec3(energy_enhance * 0.3, energy_enhance * 0.2, 0.0);
    
    // Add growth-based effects
    float growth_glow = v_growth * 0.3; // Growing resources glow more
    final_color += vec3(growth_glow * 0.2, growth_glow * 0.3, 0.0);
    
    // Add subtle sparkle for high-energy resources
    if (v_energy > 50.0) {
        float sparkle = sin(u_time * 12.0 + v_position.x * 0.5) * sin(u_time * 8.0 + v_position.y * 0.7);
        sparkle = max(0.0, sparkle * 0.4);
        final_color += vec3(sparkle * 0.6);
    }
    
    gl_FragColor = vec4(final_color, alpha);
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
            Err(format!("Failed to link resource shader program: {}", error).into())
        }
    }

    fn create_trail_shader_program(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
        let vertex_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"precision highp float;
attribute vec2 a_position;
attribute vec3 a_color;
attribute float a_life;
uniform vec2 u_canvas_size;
uniform float u_time;
varying vec3 v_color;
varying float v_life;
varying vec2 v_position;

void main() {
    vec2 ndc = (a_position / u_canvas_size) * 2.0 - 1.0;
    ndc.y = -ndc.y;
    
            gl_Position = vec4(ndc, 0.0, 1.0);
        gl_PointSize = 6.0 * (1.0 - a_life); // Increased trail size
    
    v_color = a_color;
    v_life = a_life;
    v_position = a_position;
}"#,
        )?;

        let fragment_shader = Self::create_shader(
            gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"precision highp float;
varying vec3 v_color;
varying float v_life;
varying vec2 v_position;

void main() {
    vec2 center = gl_PointCoord - 0.5;
    float dist = length(center);
    
    float alpha = 1.0 - smoothstep(0.0, 0.5, dist);
    alpha *= (1.0 - v_life); // Fade out over time
    
    vec3 final_color = v_color * (1.0 - v_life * 0.5);
    
    gl_FragColor = vec4(final_color, alpha * 0.6);
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
            Err(format!("Failed to link trail shader program: {}", error).into())
        }
    }

    fn create_shader(
        gl: &WebGlRenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, JsValue> {
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

        // Debug: Log agent count only occasionally
        if self.agent_count % 100 == 0 {
            web_sys::console::log_1(&format!("Updating {} agents", self.agent_count).into());
        }

        // Convert agents to GPU data with enhanced colors and effects
        let mut agent_data = Vec::new();
        self.agent_positions.clear();

        // Add a test particle if no agents exist
        if agents.is_empty() {
            web_sys::console::log_1(&"No agents found, adding test particle".into());
            // Add a test particle in the center
            let test_x = self.canvas_width as f32 / 2.0;
            let test_y = self.canvas_height as f32 / 2.0;

            // Position (vec2)
            agent_data.extend_from_slice(&test_x.to_le_bytes());
            agent_data.extend_from_slice(&test_y.to_le_bytes());
            self.agent_positions.push((test_x, test_y));

            // Bright red color for visibility
            agent_data.extend_from_slice(&(1.0 as f32).to_le_bytes()); // R
            agent_data.extend_from_slice(&(0.0 as f32).to_le_bytes()); // G
            agent_data.extend_from_slice(&(0.0 as f32).to_le_bytes()); // B

            // Size attribute
            agent_data.extend_from_slice(&(1.0 as f32).to_le_bytes());

            // Energy attribute
            agent_data.extend_from_slice(&(100.0 as f32).to_le_bytes());

            self.agent_count = 1;
        } else {
            for agent in agents {
                // Position (vec2)
                agent_data.extend_from_slice(&(agent.x as f32).to_le_bytes());
                agent_data.extend_from_slice(&(agent.y as f32).to_le_bytes());
                self.agent_positions.push((agent.x as f32, agent.y as f32));

                // Enhanced color based on genes and energy with predator distinction and death states
                let is_predator = agent.genes.is_predator > 0.5;

                // Handle death colors
                let (hue, saturation, lightness) = if agent.is_dying {
                    match agent.death_reason {
                        Some(DeathReason::Starvation) => (0.0, 0.8, 0.3), // Dark red for starvation
                        Some(DeathReason::OldAge) => (30.0, 0.6, 0.4), // Orange for old age
                        Some(DeathReason::KilledByPredator) => (0.0, 1.0, 0.2), // Bright red for predation
                        Some(DeathReason::Combat) => (15.0, 0.9, 0.3), // Red-orange for combat
                        Some(DeathReason::NaturalCauses) => (60.0, 0.5, 0.4), // Yellow for natural causes
                        None => (0.0, 0.7, 0.3), // Default dark red
                    }
                } else {
                    let base_hue = if is_predator {
                        // Predators: Red to orange range (0-60 degrees)
                        (agent.genes.attack_power * 60.0 + agent.genes.aggression * 30.0) % 60.0
                    } else {
                        // Prey: Blue to green range (180-240 degrees)
                        (agent.genes.speed * 60.0 + agent.genes.sense_range * 0.5 + 180.0) % 60.0
                            + 180.0
                    };

                    let base_saturation = if is_predator {
                        0.95 + agent.genes.attack_power * 0.05 // Predators more saturated
                    } else {
                        0.9 + agent.genes.size * 0.1 // Prey normal saturation
                    };

                    let base_lightness = if is_predator {
                        0.6 + agent.energy * 0.003 + agent.genes.attack_power * 0.1 // Predators brighter
                    } else {
                        0.5 + agent.energy * 0.004 // Prey normal brightness
                    };

                    (base_hue, base_saturation, base_lightness)
                };

                // Convert HSL to RGB with enhanced vibrancy
                let (r, g, b) = Self::hsl_to_rgb(hue as f32, saturation as f32, lightness as f32);

                // Add extra vibrancy and energy-based color enhancement
                let vibrancy = if is_predator { 1.5 } else { 1.3 }; // Predators more vibrant
                let energy_boost = (agent.energy * 0.002) as f32;
                let predator_boost = if is_predator {
                    agent.genes.attack_power as f32 * 0.1
                } else {
                    0.0
                };

                agent_data.extend_from_slice(
                    &(((r * vibrancy + energy_boost + predator_boost).min(1.0)) as f32)
                        .to_le_bytes(),
                );
                agent_data.extend_from_slice(
                    &(((g * vibrancy + energy_boost * 0.7 + predator_boost * 0.5).min(1.0)) as f32)
                        .to_le_bytes(),
                );
                agent_data.extend_from_slice(
                    &(((b * vibrancy + energy_boost * 0.3 + predator_boost * 0.2).min(1.0)) as f32)
                        .to_le_bytes(),
                );

                // Size attribute with fade effects
                let size_factor = if agent.is_dying {
                    1.0 - agent.death_fade as f32 // Shrink when dying
                } else {
                    agent.spawn_fade as f32 // Grow when spawning
                };
                let adjusted_size = agent.genes.size as f32 * size_factor;
                agent_data.extend_from_slice(&adjusted_size.to_le_bytes());

                // Energy attribute with fade effects
                let energy_factor = if agent.is_dying {
                    1.0 - agent.death_fade as f32 // Fade energy when dying
                } else {
                    agent.spawn_fade as f32 // Fade in energy when spawning
                };
                let adjusted_energy = agent.energy as f32 * energy_factor;
                agent_data.extend_from_slice(&adjusted_energy.to_le_bytes());
            }
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

        // Debug: Log resource count only occasionally
        if self.resource_count % 100 == 0 {
            web_sys::console::log_1(&format!("Updating {} resources", self.resource_count).into());
        }

        // Convert resources to GPU data with enhanced colors
        let mut resource_data = Vec::new();
        self.resource_positions.clear();
        self.resource_growth_states.clear();

        // Add a test resource if no resources exist
        if resources.is_empty() {
            web_sys::console::log_1(&"No resources found, adding test resource".into());
            // Add a test resource in the center
            let test_x = self.canvas_width as f32 / 2.0 + 100.0;
            let test_y = self.canvas_height as f32 / 2.0;

            // Position (vec2)
            resource_data.extend_from_slice(&test_x.to_le_bytes());
            resource_data.extend_from_slice(&test_y.to_le_bytes());
            self.resource_positions.push((test_x, test_y));

            // Bright green color for visibility
            resource_data.extend_from_slice(&(0.0 as f32).to_le_bytes()); // R
            resource_data.extend_from_slice(&(1.0 as f32).to_le_bytes()); // G
            resource_data.extend_from_slice(&(0.0 as f32).to_le_bytes()); // B

            // Energy attribute
            resource_data.extend_from_slice(&(100.0 as f32).to_le_bytes());

            // Growth attribute (fully grown)
            resource_data.extend_from_slice(&(1.0 as f32).to_le_bytes());
            self.resource_growth_states.push(1.0);

            self.resource_count = 1;
        } else {
            for (i, resource) in resources.iter().enumerate() {
                // Position (vec2)
                resource_data.extend_from_slice(&(resource.x as f32).to_le_bytes());
                resource_data.extend_from_slice(&(resource.y as f32).to_le_bytes());
                self.resource_positions
                    .push((resource.x as f32, resource.y as f32));

                // Enhanced color based on energy - vibrant green to yellow to orange
                let energy_ratio = (resource.energy / 100.0).min(1.0);
                let r = energy_ratio * 1.2;
                let g = (1.0 - energy_ratio * 0.3) * 1.1;
                let b = energy_ratio * 0.3;

                resource_data.extend_from_slice(&(r.min(1.0) as f32).to_le_bytes());
                resource_data.extend_from_slice(&(g.min(1.0) as f32).to_le_bytes());
                resource_data.extend_from_slice(&(b.min(1.0) as f32).to_le_bytes());

                // Energy attribute
                resource_data.extend_from_slice(&(resource.energy as f32).to_le_bytes());

                // Calculate growth state based on resource age and energy
                let growth_state = self.calculate_resource_growth(i, resource);
                resource_data.extend_from_slice(&growth_state.to_le_bytes());
                self.resource_growth_states.push(growth_state);
            }
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

    pub fn render(&mut self) {
        // Debug: Log render call only occasionally
        if (self.time * 60.0) as i32 % 60 == 0 {
            // Log once per second
            web_sys::console::log_1(
                &format!(
                    "WebGL Render: {} agents, {} resources, time: {:.2}",
                    self.agent_count, self.resource_count, self.time
                )
                .into(),
            );
        }

        // Update time for animations with variable speed for more dynamic effects
        self.time += 0.016; // Assuming 60 FPS

        // Create particle trails
        self.update_trails();

        // Clear the canvas with a beautiful gradient background
        self.render_background();

        // Render particle trails first (background)
        self.render_trails();

        // Render resources
        self.render_resources();

        // Render agents (foreground)
        self.render_agents();

        // Debug: Check for WebGL errors (only log once per second)
        let error = self.gl.get_error();
        if error != 0 && (self.time * 60.0) as i32 % 60 == 0 {
            web_sys::console::log_1(
                &format!("WebGL Error: {} (time: {:.2})", error, self.time).into(),
            );
        }
    }

    fn render_background(&self) {
        // Create a beautiful animated gradient background
        let time_factor = (self.time * 0.1).sin() * 0.02;
        let r = 0.1 + time_factor; // Brighter background
        let g = 0.15 + time_factor * 0.5;
        let b = 0.25 + time_factor * 0.3;

        self.gl.clear_color(r, g, b, 1.0);
        self.gl.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );
    }

    fn update_trails(&mut self) {
        // Create particle trails for agents
        let mut trail_data = Vec::new();
        let mut trail_count = 0;

        for (i, (x, y)) in self.agent_positions.iter().enumerate() {
            if i < self.agent_count as usize {
                // Create multiple trail particles per agent with more variety
                for j in 0..5 {
                    let life = (j as f32) / 5.0;
                    let offset_x = (self.time * 150.0 + i as f32 * 15.0) * 0.15;
                    let offset_y = (self.time * 120.0 + i as f32 * 12.0) * 0.15;

                    // Add some randomness to trail positions
                    let random_x = (self.time * 10.0 + i as f32 * 7.0).sin() * 2.0;
                    let random_y = (self.time * 8.0 + i as f32 * 5.0).cos() * 2.0;

                    let trail_x = x - offset_x * life + random_x;
                    let trail_y = y - offset_y * life + random_y;

                    // Position
                    trail_data.extend_from_slice(&trail_x.to_le_bytes());
                    trail_data.extend_from_slice(&trail_y.to_le_bytes());

                    // Color (faded version of agent color with more variety)
                    let hue = (i as f32 * 45.0 + j as f32 * 20.0) % 360.0;
                    let (r, g, b) = Self::hsl_to_rgb(hue, 0.8, 0.7);
                    trail_data.extend_from_slice(&(r * (1.0 - life * 0.7)).to_le_bytes());
                    trail_data.extend_from_slice(&(g * (1.0 - life * 0.7)).to_le_bytes());
                    trail_data.extend_from_slice(&(b * (1.0 - life * 0.7)).to_le_bytes());

                    // Life
                    trail_data.extend_from_slice(&life.to_le_bytes());

                    trail_count += 1;
                }
            }
        }

        self.trail_count = trail_count;

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.trail_buffer),
        );
        self.gl.buffer_data_with_u8_array(
            WebGlRenderingContext::ARRAY_BUFFER,
            &trail_data,
            WebGlRenderingContext::DYNAMIC_DRAW,
        );
    }

    fn render_trails(&self) {
        if self.trail_count == 0 {
            return;
        }

        // Clear any previous errors
        self.gl.get_error();

        self.gl.use_program(Some(&self.trail_program));

        // Set uniforms for trail program
        let canvas_size_location = self
            .gl
            .get_uniform_location(&self.trail_program, "u_canvas_size");
        let time_location = self.gl.get_uniform_location(&self.trail_program, "u_time");

        if let Some(ref location) = canvas_size_location {
            self.gl.uniform2f(
                Some(location),
                self.canvas_width as f32,
                self.canvas_height as f32,
            );
        }
        if let Some(ref location) = time_location {
            self.gl.uniform1f(Some(location), self.time);
        }

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.trail_buffer),
        );

        // Position attribute (vec2)
        let position_location =
            self.gl
                .get_attrib_location(&self.trail_program, "a_position") as u32;
        self.gl.enable_vertex_attrib_array(position_location);
        self.gl.vertex_attrib_pointer_with_i32(
            position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            24, // 2 floats for position + 3 floats for color + 1 float for life
            0,
        );

        // Color attribute (vec3)
        let color_location = self.gl.get_attrib_location(&self.trail_program, "a_color") as u32;
        self.gl.enable_vertex_attrib_array(color_location);
        self.gl.vertex_attrib_pointer_with_i32(
            color_location,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            24,
            8,
        );

        // Life attribute (float)
        let life_location = self.gl.get_attrib_location(&self.trail_program, "a_life") as u32;
        self.gl.enable_vertex_attrib_array(life_location);
        self.gl.vertex_attrib_pointer_with_i32(
            life_location,
            1,
            WebGlRenderingContext::FLOAT,
            false,
            24,
            20,
        );

        // Draw points
        self.gl
            .draw_arrays(WebGlRenderingContext::POINTS, 0, self.trail_count as i32);
    }

    fn render_resources(&self) {
        if self.resource_count == 0 {
            return;
        }

        // Clear any previous errors
        self.gl.get_error();

        self.gl.use_program(Some(&self.resource_program));

        // Set uniforms for resource program
        let canvas_size_location = self
            .gl
            .get_uniform_location(&self.resource_program, "u_canvas_size");
        let time_location = self
            .gl
            .get_uniform_location(&self.resource_program, "u_time");

        if let Some(ref location) = canvas_size_location {
            self.gl.uniform2f(
                Some(location),
                self.canvas_width as f32,
                self.canvas_height as f32,
            );
        }
        if let Some(ref location) = time_location {
            self.gl.uniform1f(Some(location), self.time);
        }

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.resource_buffer),
        );

        // Position attribute (vec2)
        let position_location =
            self.gl
                .get_attrib_location(&self.resource_program, "a_position") as u32;
        self.gl.enable_vertex_attrib_array(position_location);
        self.gl.vertex_attrib_pointer_with_i32(
            position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            28, // 2 floats for position + 3 floats for color + 1 float for energy + 1 float for growth
            0,
        );

        // Color attribute (vec3)
        let color_location =
            self.gl
                .get_attrib_location(&self.resource_program, "a_color") as u32;
        self.gl.enable_vertex_attrib_array(color_location);
        self.gl.vertex_attrib_pointer_with_i32(
            color_location,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            28,
            8,
        );

        // Energy attribute (float)
        let energy_location =
            self.gl
                .get_attrib_location(&self.resource_program, "a_energy") as u32;
        self.gl.enable_vertex_attrib_array(energy_location);
        self.gl.vertex_attrib_pointer_with_i32(
            energy_location,
            1,
            WebGlRenderingContext::FLOAT,
            false,
            28,
            20,
        );

        // Growth attribute (float)
        let growth_location =
            self.gl
                .get_attrib_location(&self.resource_program, "a_growth") as u32;
        self.gl.enable_vertex_attrib_array(growth_location);
        self.gl.vertex_attrib_pointer_with_i32(
            growth_location,
            1,
            WebGlRenderingContext::FLOAT,
            false,
            28,
            24,
        );

        // Draw points
        self.gl
            .draw_arrays(WebGlRenderingContext::POINTS, 0, self.resource_count as i32);
    }

    fn render_agents(&self) {
        if self.agent_count == 0 {
            return;
        }

        // Clear any previous errors
        self.gl.get_error();

        self.gl.use_program(Some(&self.agent_program));

        // Set uniforms
        if let Some(ref location) = self.canvas_size_location {
            self.gl.uniform2f(Some(location), 800.0, 600.0);
        }
        if let Some(ref location) = self.time_location {
            self.gl.uniform1f(Some(location), self.time);
        }

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.agent_buffer),
        );

        // Position attribute (vec2)
        let position_location =
            self.gl
                .get_attrib_location(&self.agent_program, "a_position") as u32;
        self.gl.enable_vertex_attrib_array(position_location);
        self.gl.vertex_attrib_pointer_with_i32(
            position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            32, // 2 floats for position + 3 floats for color + 1 float for size + 1 float for energy
            0,
        );

        // Color attribute (vec3)
        let color_location = self.gl.get_attrib_location(&self.agent_program, "a_color") as u32;
        self.gl.enable_vertex_attrib_array(color_location);
        self.gl.vertex_attrib_pointer_with_i32(
            color_location,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            32,
            8,
        );

        // Size attribute (float)
        let size_location = self.gl.get_attrib_location(&self.agent_program, "a_size") as u32;
        self.gl.enable_vertex_attrib_array(size_location);
        self.gl.vertex_attrib_pointer_with_i32(
            size_location,
            1,
            WebGlRenderingContext::FLOAT,
            false,
            32,
            20,
        );

        // Energy attribute (float)
        let energy_location = self.gl.get_attrib_location(&self.agent_program, "a_energy") as u32;
        self.gl.enable_vertex_attrib_array(energy_location);
        self.gl.vertex_attrib_pointer_with_i32(
            energy_location,
            1,
            WebGlRenderingContext::FLOAT,
            false,
            32,
            24,
        );

        // Draw points
        self.gl
            .draw_arrays(WebGlRenderingContext::POINTS, 0, self.agent_count as i32);
    }

    fn calculate_resource_growth(&self, index: usize, resource: &Resource) -> f32 {
        // Calculate growth state based on energy and fade states
        let base_growth = (resource.energy / 100.0).min(1.0) as f32;
        
        // Apply spawn fade-in
        let spawn_factor = if resource.is_spawning {
            resource.spawn_fade as f32
        } else {
            1.0
        };
        
        // Apply depletion fade-out
        let deplete_factor = if resource.is_depleting {
            1.0 - resource.deplete_fade as f32
        } else {
            1.0
        };
        
        // Combine all factors
        let growth_state = base_growth * spawn_factor * deplete_factor;
        
        // Ensure minimum visibility during spawn
        growth_state.max(0.1)
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

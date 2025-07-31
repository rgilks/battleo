use crate::simulation_core::{SimulationConfig, UnifiedSimulation};
use crate::webgl_renderer::WebGlRenderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct WebSimulation {
    simulation: UnifiedSimulation,
    canvas: HtmlCanvasElement,
    webgl_renderer: Option<WebGlRenderer>,
    is_running: bool,
    use_webgl: bool,
    last_frame_time: f64,
    frame_count: u32,
}

#[wasm_bindgen]
impl WebSimulation {
    pub fn new(canvas_id: &str) -> Result<WebSimulation, JsValue> {
        console_error_panic_hook::set_once();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;

        // Set canvas size based on available space
        let window = web_sys::window().unwrap();
        let screen_width = window.inner_width().unwrap().as_f64().unwrap();
        let screen_height = window.inner_height().unwrap().as_f64().unwrap();

        // Calculate canvas size (leave space for sidebar)
        let canvas_width = (screen_width - 250.0).max(800.0) as u32;
        let canvas_height = (screen_height - 100.0).max(600.0) as u32;

        canvas.set_width(canvas_width);
        canvas.set_height(canvas_height);

        // Create simulation configuration
        let config = SimulationConfig {
            width: canvas_width as f64,
            height: canvas_height as f64,
            max_agents: 10000,
            max_resources: 1500,
            initial_agents: 500,
            initial_resources: 500,
            resource_spawn_rate: 0.2,
            target_duration_minutes: 5.0,
            stability_threshold: 0.1,
            min_agent_count: 10,
            max_agent_count: 3000,
            use_ecs: true,
        };

        // Create simulation
        let simulation = UnifiedSimulation::new(config);

        // Try to initialize WebGL
        let webgl_renderer = match WebGlRenderer::new(canvas.clone()) {
            Ok(renderer) => Some(renderer),
            Err(e) => {
                web_sys::console::log_1(&format!("WebGL initialization failed: {:?}", e).into());
                None
            }
        };
        let use_webgl = webgl_renderer.is_some();

        if use_webgl {
            web_sys::console::log_1(&"WebGL initialized successfully!".into());
        } else {
            web_sys::console::log_1(&"Using Canvas 2D rendering (WebGL not available)".into());
        }

        Ok(WebSimulation {
            simulation,
            canvas,
            webgl_renderer,
            is_running: false,
            use_webgl,
            last_frame_time: 0.0,
            frame_count: 0,
        })
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.is_running = true;
            self.animate();
        }
    }

    pub fn stop(&mut self) {
        self.is_running = false;
    }

    pub fn step(&mut self) {
        self.simulation.update();
        self.render();
    }

    pub fn get_stats(&self) -> JsValue {
        let stats = self.simulation.get_stats();
        serde_wasm_bindgen::to_value(&stats).unwrap()
    }

    pub fn get_rendering_mode(&self) -> String {
        if self.use_webgl {
            "WebGL".to_string()
        } else {
            "Canvas 2D".to_string()
        }
    }

    pub fn is_rayon_available(&self) -> bool {
        UnifiedSimulation::is_rayon_available()
    }

    pub fn set_rayon_initialized(&self, initialized: bool) {
        UnifiedSimulation::set_rayon_initialized(initialized);
    }

    pub fn force_webgl(&mut self) -> bool {
        // Try to force WebGL initialization
        match WebGlRenderer::new(self.canvas.clone()) {
            Ok(renderer) => {
                self.webgl_renderer = Some(renderer);
                self.use_webgl = true;
                web_sys::console::log_1(&"WebGL forced successfully!".into());
                true
            }
            Err(e) => {
                web_sys::console::log_1(&format!("Failed to force WebGL: {:?}", e).into());
                false
            }
        }
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        self.simulation.add_agent(x, y);
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        self.simulation.add_resource(x, y);
    }

    pub fn reset(&mut self) {
        self.simulation.reset();
    }

    pub fn animate(&mut self) {
        if !self.is_running {
            return;
        }

        self.simulation.update();
        self.render();

        // Request next frame
        let window = web_sys::window().unwrap();
        let _ = window.request_animation_frame(
            Closure::wrap(Box::new(move |_| {
                // This will be handled by the animation loop
            }) as Box<dyn FnMut(f64)>)
            .into_js_value()
            .unchecked_ref(),
        );
    }

    fn render(&mut self) {
        if self.use_webgl {
            self.render_webgl();
        } else {
            self.render_canvas2d();
        }
    }

    fn render_webgl(&mut self) {
        if let Some(ref mut renderer) = self.webgl_renderer {
            // Get agents and resources from unified simulation
            let agents = self.simulation.get_agents();
            let resources = self.simulation.get_resources();

            renderer.update_agents(&agents);
            renderer.update_resources(&resources);
            renderer.render();

            // Debug: Log rendering info only occasionally
            self.frame_count += 1;
            if self.frame_count % 60 == 0 {
                // Log once per second at 60fps
                        // Removed verbose logging
            }
        } else {
            web_sys::console::log_1(&"WebGL renderer is None!".into());
        }
    }

    fn render_canvas2d(&self) {
        // Fallback to Canvas 2D rendering
        let ctx = match self.canvas.get_context("2d") {
            Ok(Some(context)) => match context.dyn_into::<CanvasRenderingContext2d>() {
                Ok(ctx) => ctx,
                Err(_) => return,
            },
            _ => return,
        };

        // Clear canvas
        ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // Render background
        ctx.set_fill_style(&"#1a1a2e".into());
        ctx.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );

        // Render resources
        let resources = self.simulation.get_resources();
        for resource in &resources {
            ctx.set_fill_style(&format!("hsl({}, 70%, 60%)", resource.energy * 120.0).into());
            ctx.begin_path();
            ctx.arc(
                resource.x,
                resource.y,
                resource.size,
                0.0,
                2.0 * std::f64::consts::PI,
            )
            .unwrap();
            ctx.fill();
        }

        // Render agents
        let agents = self.simulation.get_agents();
        for agent in &agents {
            let hue = (agent.genes.speed * 100.0 + agent.genes.sense_range * 50.0) % 360.0;
            let saturation = 70.0 + agent.genes.size * 20.0;
            let lightness = 50.0 + agent.energy * 20.0;

            ctx.set_fill_style(&format!("hsl({}, {}%, {}%)", hue, saturation, lightness).into());
            ctx.begin_path();
            ctx.arc(
                agent.x,
                agent.y,
                agent.genes.size * 3.0,
                0.0,
                2.0 * std::f64::consts::PI,
            )
            .unwrap();
            ctx.fill();

            // Draw direction indicator
            ctx.set_stroke_style(&"#ffffff".into());
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(agent.x, agent.y);
            ctx.line_to(
                agent.x + agent.dx * agent.genes.size * 4.0,
                agent.y + agent.dy * agent.genes.size * 4.0,
            );
            ctx.stroke();
        }
    }
}

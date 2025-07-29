use rand::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

mod agent;
mod genes;
mod resource;
mod simulation;
mod webgl_renderer;

use simulation::Simulation;
use webgl_renderer::WebGlRenderer;

#[wasm_bindgen]
pub struct BattleSimulation {
    simulation: Simulation,
    canvas: HtmlCanvasElement,
    webgl_renderer: Option<WebGlRenderer>,
    is_running: bool,
    use_webgl: bool,
}

#[wasm_bindgen]
impl BattleSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<BattleSimulation, JsValue> {
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

        // Create simulation with canvas dimensions
        let simulation = Simulation::new(canvas_width as f64, canvas_height as f64);

        // Try to initialize WebGL
        web_sys::console::log_1(&"Attempting to initialize WebGL...".into());
        let webgl_renderer = match WebGlRenderer::new(canvas.clone()) {
            Ok(renderer) => {
                web_sys::console::log_1(&"WebGL renderer created successfully!".into());
                Some(renderer)
            }
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

        Ok(BattleSimulation {
            simulation,
            canvas,
            webgl_renderer,
            is_running: false,
            use_webgl,
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
            renderer.update_agents(&self.simulation.agents);
            renderer.update_resources(&self.simulation.resources);
            renderer.render();
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
        for resource in &self.simulation.resources {
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
        for agent in &self.simulation.agents {
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

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

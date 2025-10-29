use eframe::egui;
use egui::{Pos2, Vec2};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]


pub struct Simulation {
    num_points: usize,
    num_types: usize,
    colors: Vec<egui::Color32>,
    #[serde(skip)]
    points: Vec<Particle>,
    #[serde(skip)]
    last_time: std::time::Instant,
    #[serde(skip)]
    fps: f32,
    matrix: Vec<Vec<f32>>,

}

#[derive(Clone, Copy)]
struct Particle {
    position: Pos2,
    velocity: Vec2,
    color: usize
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            num_points: 200,
            num_types: 2,
            colors: Vec::new(),
            points: Vec::new(),
            last_time: std::time::Instant::now(),
            fps: 0.0,
            matrix: Vec::new()
        }
    }
}

impl Simulation {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    /// Genera un insieme di particelle casuali all’interno dello schermo
    fn generate_points(&mut self, width: f32, height: f32) {
        self.num_types = 2;
        use rand::Rng;
        let mut rng = rand::rng();
        self.points = (0..self.num_points)
            .map(|_| Particle {
                position: Pos2::new(rng.random_range(0.0..width), rng.random_range(0.0..height)),
                velocity: Vec2::new(0.0, 0.0),
                color: rng.random_range(0..self.num_types)
            })
            .collect();
        self.colors.clear();
        for _i in 0..self.num_types {
            self.colors.push(egui::Color32::from_rgb(rng.random_range(0..255), rng.random_range(0..255), rng.random_range(0..255)));
        }
        println!("{}", self.num_types);
        println!("{}", self.colors.len());

        self.matrix = (0..self.num_types)
        .map(|_| {
            (0..self.num_types)
                .map(|_| rng.random_range(-1.0..1.0))
                .collect()
        })
        .collect();
    }

    /// Aggiorna la fisica della simulazione (repulsione tra particelle)
    fn update_physics(&mut self) {
        let len = self.num_points;
        for _i in 0..len {
            let p1 = self.points[_i];
            let mut sum: Vec2 = Vec2::new(0.0, 0.0);
            for _j in 0..len{

                let p2 = self.points[_j];
                let d = egui::Vec2::new(p2.position.x - p1.position.x, p2.position.y - p1.position.y);
                let dist = d.length().max(0.01);
                if dist > 10.0 {continue;}
                let f = Simulation::force(dist / 10.0, self.matrix[p1.color][p2.color]);
                
                let unit_d = Vec2::new(d.x / dist, d.y / dist);

                sum.x += unit_d.x * f;
                sum.y += unit_d.y * f;
            }
            sum.x *= 10.0;
            sum.y *= 10.0;

            let now = std::time::Instant::now();
            let dt = now.duration_since(self.last_time).as_secs_f32();
            let p = &mut self.points[_i];
            p.velocity = p.velocity + sum * dt;
            p.position += p.velocity * dt;
        }
    }

    pub fn force(r: f32, a: f32) -> f32{
        let beta = 0.3;
        if r < beta{
            return r / beta - 1.0;
        }
        else if beta < r  && r < 1.0 {
            return a * (1.0 - (2.0 * r - 1.0 - beta).abs() / (1.0 - beta));
        }else{
            return 0.0;
        }
    }
}

impl eframe::App for Simulation {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Finestra controlli
        egui::Window::new("Controlli")
            .resizable(true)
            .default_size([200.0, 120.0])
            .show(ctx, |ui| {
                let rect = ctx.content_rect();
                let width = rect.width();
                let height = rect.height();

                ui.label("Numero di pallini:");
                if ui
                    .add(egui::Slider::new(&mut self.num_points, 10..=2000))
                    .changed()
                {
                    self.generate_points(width, height);
                }

                if ui.button("Rigenera").clicked() {
                    self.generate_points(width, height);
                }

                // FPS
                let now = std::time::Instant::now();
                let dt = now.duration_since(self.last_time).as_secs_f32();
                self.last_time = now;
                if dt > 0.0 {
                    self.fps = 1.0 / dt;
                }

                ui.label(format!("FPS: {:.1}", self.fps));
            });

        // Aggiorna la fisica
        if  self.points.len() > 0 {
            self.update_physics();
        }

        // Disegna le particelle
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            for p in &self.points {
                painter.circle_filled(p.position, 3.0, self.colors[p.color]);
            }
        });

        // Richiedi un nuovo frame continuo
        ctx.request_repaint();
    }
}

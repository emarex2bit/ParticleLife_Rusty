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
    radius: f32,
    width: f32,
    height: f32,
    coeff_friction: f32,

}

#[derive(Clone, Copy)]
struct Particle {
    position: Pos2,
    velocity: Vec2,
    color: usize,
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
            matrix: Vec::new(),
            radius: 50.0,
            width: 1920.0,
            height: 1080.0,
            coeff_friction: 0.005
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
        use rand::Rng;
        let mut rng = rand::rng();
        self.points = (0..self.num_points)
            .map(|_| Particle {
                position: Pos2::new(rng.random_range((width / 3.0)..(width / 3.0 * 2.0)), rng.random_range((height / 3.0)..(height / 3.0 * 2.0))),
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

    fn update_physics(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f32();
        self.last_time = now;
    
        let len = self.num_points;
    
        // Bordo toroidale
        let min_x: f32 = self.width / 3.0;
        let max_x: f32 = self.width / 3.0 * 2.0;
        let min_y: f32 = self.height / 3.0;
        let max_y: f32 = self.height / 3.0 * 2.0;
        let width = max_x - min_x;
        let height = max_y - min_y;
    
        for i in 0..len {
            let p1 = self.points[i];
            let mut sum = Vec2::ZERO;
    
            for j in 0..len {
                if i == j { continue; }
                let p2 = self.points[j];
    
                // --- DIFFERENZA TOROIDALE ---
                let mut dx = p2.position.x - p1.position.x;
                let mut dy = p2.position.y - p1.position.y;
    
                // Avvolgimento toroidale delle distanze
                if dx.abs() > width / 2.0 {
                    dx -= width * dx.signum();
                }
                if dy.abs() > height / 2.0 {
                    dy -= height * dy.signum();
                }
    
                let d = Vec2::new(dx, dy);
                let dist = d.length().max(0.00001);
                if dist > self.radius { continue; }
    
                let f = Simulation::force(dist / self.radius, self.matrix[p1.color][p2.color]);
                sum += d / dist * f;
            }
    
            sum *= self.radius;
    
            let p = &mut self.points[i];
            p.velocity = 0.5_f32.powf(dt / self.coeff_friction) * p.velocity + sum * dt;
            p.position += p.velocity * dt;
    
            // --- WRAP POSIZIONE TOROIDALE ---
            if p.position.x < min_x { p.position.x += width; }
            if p.position.x > max_x { p.position.x -= width; }
            if p.position.y < min_y { p.position.y += height; }
            if p.position.y > max_y { p.position.y -= height; }
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
                self.width = rect.width();
                self.height = rect.height();

                ui.label("Numero di pallini:");
                if ui
                    .add(egui::Slider::new(&mut self.num_points, 10..=2000))
                    .changed()
                {
                    self.generate_points(self.width, self.height);
                }
                ui.label("Raggio:");
                ui.add(egui::Slider::new(&mut self.radius, 1.0..=500.0));
                ui.label("Num. Types:");
                if ui.add(egui::Slider::new(&mut self.num_types, 1..=50)).changed() {
                    self.generate_points(self.width, self.height);
                }
                ui.label("Coefficente Attrito:");
                ui.add(egui::Slider::new(&mut self.coeff_friction, 0.0001..=0.1));
                if ui.button("Rigenera").clicked() {
                    println!("{}, {}", self.width, self.height);
                    self.generate_points(self.width, self.height);
                }

                if self.matrix.is_empty() {
                    ui.label("Genera prima le particelle per creare la matrice.");
                    return;
                }
                ui.heading("Matrice Legami");
                ui.label("Clicca e modifica i valori dei legami tra tipi di particelle:");
                ui.separator();

                egui::Grid::new("matrix_editor_grid")
                    .spacing([8.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Intestazioni colonne
                        ui.label("");
                        for j in 0..self.num_types {
                            ui.label(format!("T{}", j));
                        }
                        ui.end_row();

                        // Righe
                        for i in 0..self.num_types {
                            ui.label(format!("T{}", i)); // intestazione riga
                            for j in 0..self.num_types {
                                ui.add(
                                    egui::DragValue::new(&mut self.matrix[i][j])
                                        .speed(0.01)
                                        .range(-1.0..=1.0)
                                );
                            }
                            ui.end_row();
                        }
                    });

                if ui.button("Randomizza legami").clicked() {
                    use rand::Rng;
                    let mut rng = rand::rng();
                    for i in 0..self.num_types {
                        for j in 0..self.num_types {
                            self.matrix[i][j] = rng.random_range(-1.0..1.0);
                        }
                    }
                }

                // FPS
                let now = std::time::Instant::now();
                let dt = now.duration_since(self.last_time).as_secs_f32();
                if dt > 0.0 {
                    self.fps = 1.0 / dt;
                }

                ui.label(format!("FPS: {:.1}", self.fps));
            });

        // Aggiorna la fisica
        if  self.points.len() > 0 {
            self.update_physics();
        }

        // Disegna le particelle (con replica toroidale)
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
        
            let min_x = self.width / 3.0;
            let max_x = self.width / 3.0 * 2.0;
            let min_y = self.height / 3.0;
            let max_y = self.height / 3.0 * 2.0;
            let width = max_x - min_x;
            let height = max_y - min_y;
        
            for p in &self.points {
                let base = p.position;
        
                // Genera tutte le 9 posizioni toroidali (centro + 8 repliche)
                for dx in [-width, 0.0, width] {
                    for dy in [-height, 0.0, height] {
                        let pos = Pos2::new(base.x + dx, base.y + dy);
                        painter.circle_filled(pos, 3.0, self.colors[p.color]);
                    }
                }
            }
        
            // Disegna il riquadro di riferimento
            //let stroke = egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE);
            //painter.line_segment([Pos2::new(min_x, 0.0), Pos2::new(min_x, self.height)], stroke);
            //painter.line_segment([Pos2::new(max_x, 0.0), Pos2::new(max_x, self.height)], stroke);
            //painter.line_segment([Pos2::new(0.0, min_y), Pos2::new(self.width, min_y)], stroke);
            //painter.line_segment([Pos2::new(0.0, max_y), Pos2::new(self.width, max_y)], stroke);
        });
        


        // Richiedi un nuovo frame continuo
        ctx.request_repaint();
    }
}


struct Drag{
    dragging:bool,
    x:f32,
    y:f32,
    x2:f32,
    y2:f32,
}

enum Mode{
    Play,
    Edit,
}

struct Rect{
    x:f32,
    y:f32,
    width:f32,
    height:f32,    
}

impl Rect{
    fn contains(&self, point:[f32;2]) -> bool{
        point[0] > self.x && point[1] > self.y && point[0] < self.x+self.width && point[1] < self.y+self.height
    }

    fn center(&self) -> [f32;2]{
        [self.x+self.width/2.0, self.y+self.height/2.0]
    }

    fn overlaps(a:&Rect, b:&Rect) -> bool{
        let center_a = a.center();
        let center_b = b.center();
        let center_dist_x = (center_a[0] - center_b[0]).abs();
        let center_dist_y = (center_a[1] - center_b[1]).abs();
        let radius_dist_x = (a.width + b.width) / 2.0;
        let radius_dist_y = (a.height + b.height)/ 2.0;
        center_dist_x < radius_dist_x && center_dist_y < radius_dist_y
    }
}

struct Player{
    active:bool,
    rect:Rect,
    velocity_x:f32,
    velocity_y:f32,
    grounded:bool,
}


pub struct VectorGraphics {
    blocks:Vec<Rect>,
    player:Player,
    drag:Drag,
    mouse_position:[f32;2],
    mode:Mode,
    gravity:f32,
    player_speed:f32,
    jump_force:f32,
    left_arrow:bool,
    right_arrow:bool,
    up_arrow:bool,
}

impl VectorGraphics {
    fn abs_rect(x:f32, y:f32, w:f32, h:f32) -> (f32, f32, f32, f32){
        let mut result_x = x;
        let mut result_y = y;
        let mut result_w = w;
        let mut result_h = h;
        if w<0.0{
            result_x = x + w;
            result_w = -w;
        }
        if h<0.0{
            result_y = y + h;
            result_h = -h;
        }
        (result_x, result_y, result_w, result_h)
    }

    fn find_block_at_point(&self, point:[f32;2]) -> Option<usize>{
        for i in 0..self.blocks.len(){
            if self.blocks[i].contains(point){
                return Some(i);
            }
        }
        None
    }

    pub fn new() -> VectorGraphics{
        return VectorGraphics { 
            blocks:Vec::new(), 
            player:Player { 
                active: false, 
                rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }, 
                velocity_x: 0.0, 
                velocity_y: 0.0, 
                grounded: true 
            },
            drag:Drag { dragging: false, x: 0.0, y: 0.0, x2: 0.0, y2: 0.0 }, 
            mouse_position:[0.0,0.0],
            mode:Mode::Edit,
            gravity:0.3,
            player_speed:5.0,
            jump_force: 14.0,
            left_arrow:false,
            right_arrow:false,
            up_arrow:false,
         };
    }

    pub fn mousemove(&mut self, mouse_position:[f32;2]){
        self.mouse_position = mouse_position;
        if self.drag.dragging {
            self.drag.x2 = mouse_position[0];
            self.drag.y2 = mouse_position[1];
        }
    }

    pub fn keydown(&mut self, key:winit::keyboard::KeyCode){
        match key {
            winit::keyboard::KeyCode::ArrowLeft=>{ self.left_arrow = true }
            winit::keyboard::KeyCode::ArrowRight=>{ self.right_arrow = true } 
            winit::keyboard::KeyCode::ArrowUp=>{ self.up_arrow = true }
            _ => {}
        }
        match  self.mode {
            Mode::Play => {
                match key {
                    winit::keyboard::KeyCode::Escape=>{
                        self.mode = Mode::Edit;
                    }
                    _ => {}
                }
            }
            Mode::Edit => {
                match key{
                    winit::keyboard::KeyCode::Backspace=>{
                        match self.find_block_at_point(self.mouse_position){
                            Some(id) => {self.blocks.remove(id);}
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyR=>{
                        if !self.drag.dragging{
                            self.drag.x = self.mouse_position[0];
                            self.drag.y = self.mouse_position[1];
                            self.drag.x2 = self.mouse_position[0];
                            self.drag.y2 = self.mouse_position[1];
                            self.drag.dragging = true;
                        }
                    }
                    winit::keyboard::KeyCode::KeyP=>{
                        match self.find_block_at_point(self.mouse_position){
                            Some(id) => {
                                self.player.rect = self.blocks.remove(id);
                                self.player.velocity_x = 0.0;
                                self.player.velocity_y = 0.0;
                                self.player.grounded = false;
                                self.player.active = true;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::Escape=>{
                        self.mode = Mode::Play;
                    }
                    _=>{}
                }
            }
        }
    }

    pub fn keyup(&mut self, key:winit::keyboard::KeyCode){
        match key{
            winit::keyboard::KeyCode::ArrowLeft=>{ self.left_arrow = false }
            winit::keyboard::KeyCode::ArrowRight=>{ self.right_arrow = false } 
            winit::keyboard::KeyCode::ArrowUp=>{ self.up_arrow = false }
            winit::keyboard::KeyCode::KeyR=>{
                if self.drag.dragging{
                    self.drag.dragging = false;
                    let abs_rect = Self::abs_rect(self.drag.x, self.drag.y, self.drag.x2 - self.drag.x, self.drag.y2 - self.drag.y);
                    self.blocks.push(Rect {
                        x: abs_rect.0, 
                        y: abs_rect.1, 
                        width: abs_rect.2, 
                        height: abs_rect.3,
                    });
                }
            }
            _=>{}
        }
    }

    fn overlaps_any_block(&self, rect:&Rect)->bool{
        for o in &self.blocks{
            if Rect::overlaps(rect, o){
                return true;
            }
        }
        return false;
    }

    fn slide_x(&mut self, distance:f32){
        self.player.rect.x += distance;
        if self.overlaps_any_block(&self.player.rect) { 
            self.player.rect.x -= distance;
            self.player.velocity_x = 0.0;
        }
    }

    fn slide_y(&mut self, distance:f32){
        self.player.rect.y += distance;
        if self.overlaps_any_block(&self.player.rect){
            self.player.rect.y -= distance;
            if distance>=0.0 {
                self.player.grounded = true;
            }
           self.player.velocity_y = 0.0;
        }
    }

    pub fn update(&mut self, mesh:&mut crate::mesh::Mesh, queue:&wgpu::Queue){
        if self.drag.dragging {
            let abs_rect = Self::abs_rect(self.drag.x, self.drag.y, self.drag.x2 - self.drag.x, self.drag.y2 - self.drag.y);
            mesh.add_rect(abs_rect.0, abs_rect.1, abs_rect.2, abs_rect.3, 0.0, 0.0, 1.0);
        }
        match self.mode{
            Mode::Play => {
                if self.player.active{
                    self.player.velocity_y += self.gravity;
                    self.slide_x(self.player.velocity_x);
                    self.slide_y(self.player.velocity_y);
                    if self.left_arrow{
                        self.slide_x(-self.player_speed);
                    }
                    if self.right_arrow{
                        self.slide_x(self.player_speed);
                    }
                    if self.up_arrow && self.player.grounded{
                        self.player.velocity_y -= self.jump_force;
                    }
                    self.player.grounded = false;
                }
            }
            Mode::Edit => {}
        }
        for block in &self.blocks{
            mesh.add_rect(block.x, block.y, block.width, block.height, 0.025, 0.025, 0.025);
        }
        if self.player.active{
            mesh.add_rect(self.player.rect.x, self.player.rect.y, self.player.rect.width, self.player.rect.height, 1.0, 0.5, 0.0);
        }
        mesh.update_queue(queue);
    }
}
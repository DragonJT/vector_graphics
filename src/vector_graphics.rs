
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

enum ObjectType{
    Player,
    Block,
}

struct Object{
    x:f32,
    y:f32,
    width:f32,
    height:f32,
    object_type:ObjectType,
    velocity_x:f32,
    velocity_y:f32,
    grounded:bool,
}


impl Object{
    fn contains(&self, point:[f32;2]) -> bool{
        point[0] > self.x && point[1] > self.y && point[0] < self.x+self.width && point[1] < self.y+self.height
    }

    fn center(&self) -> [f32;2]{
        [self.x+self.width/2.0, self.y+self.height/2.0]
    }

    fn overlaps(a:&Object, b:&Object) -> bool{
        let center_a = a.center();
        let center_b = b.center();
        let center_dist_x = (center_a[0] - center_b[0]).abs();
        let center_dist_y = (center_a[1] - center_b[1]).abs();
        let radius_dist_x = (a.width + b.width) / 2.0;
        let radius_dist_y = (a.height + b.height)/ 2.0;
        center_dist_x < radius_dist_x && center_dist_y < radius_dist_y
    }
}

pub struct VectorGraphics {
    objects:Vec<Object>,
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

    fn find_object_at_point(&self, point:[f32;2]) -> Option<usize>{
        for i in 0..self.objects.len(){
            if self.objects[i].contains(point){
                return Some(i);
            }
        }
        None
    }

    pub fn new() -> VectorGraphics{
        return VectorGraphics { 
            objects:Vec::new(), 
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
                        match self.find_object_at_point(self.mouse_position){
                            Some(id) => {self.objects.remove(id);}
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
                        match self.find_object_at_point(self.mouse_position){
                            Some(id) => {self.objects[id].object_type = ObjectType::Player}
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
                    self.objects.push(Object {
                        x: abs_rect.0, 
                        y: abs_rect.1, 
                        width: abs_rect.2, 
                        height: abs_rect.3,
                        object_type:ObjectType::Block,
                        velocity_x: 0.0,
                        velocity_y: 0.0,
                        grounded:false,
                    });
                }
            }
            _=>{}
        }
    }

    fn overlaps_any_block(&self, object:&Object)->bool{
        for o in &self.objects{
            match o.object_type {
                ObjectType::Block => {
                    if Object::overlaps(object, o){
                        return true;
                    }
                }
                _=>{}
            }
        }
        return false;
    }

    fn slide_x(&mut self, i:usize, distance:f32){
        self.objects[i].x += distance;
        if self.overlaps_any_block(&self.objects[i]) { 
            self.objects[i].x -= distance;
            self.objects[i].velocity_x = 0.0;
        }
    }

    fn slide_y(&mut self, i:usize, distance:f32){
        self.objects[i].y += distance;
        if self.overlaps_any_block(&self.objects[i]){
            self.objects[i].y -= distance;
            if distance>=0.0 {
                self.objects[i].grounded = true;
            }
            self.objects[i].velocity_y = 0.0;
        }
    }

    pub fn update(&mut self, mesh:&mut crate::mesh::Mesh, queue:&wgpu::Queue){
        if self.drag.dragging {
            let abs_rect = Self::abs_rect(self.drag.x, self.drag.y, self.drag.x2 - self.drag.x, self.drag.y2 - self.drag.y);
            mesh.add_rect(abs_rect.0, abs_rect.1, abs_rect.2, abs_rect.3, 0.0, 0.0, 1.0);
        }
        match self.mode{
            Mode::Play => {
                for i in 0..self.objects.len(){
                    match self.objects[i].object_type{
                        ObjectType::Block => {}
                        ObjectType::Player => {
                            self.objects[i].velocity_y += self.gravity;
                            self.slide_x(i, self.objects[i].velocity_x);
                            self.slide_y(i, self.objects[i].velocity_y);
                            if self.left_arrow{
                                self.slide_x(i, -self.player_speed);
                            }
                            if self.right_arrow{
                                self.slide_x(i, self.player_speed);
                            }
                            if self.up_arrow && self.objects[i].grounded{
                                self.objects[i].velocity_y -= self.jump_force;
                            }
                            self.objects[i].grounded = false;
                        }
                    }
                }
            }
            Mode::Edit => {}
        }
        for object in &self.objects{
            let (r,g,b) = match &object.object_type {
                ObjectType::Block => (0.025,0.025,0.025),
                ObjectType::Player => (1.0, 0.5, 0.0),
            };
            mesh.add_rect(object.x, object.y, object.width, object.height, r, g, b);
        }
        mesh.update_queue(queue);
    }
}
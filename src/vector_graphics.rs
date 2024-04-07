
struct Drag{
    dragging:bool,
    x:f32,
    y:f32,
    x2:f32,
    y2:f32,
}

enum CollisionType{
    Bounce,
    PortalTo(usize),
    None,
}

enum Mode{
    Play,
    Edit,
}

enum ObjectType{
    Player,
    Enemy,
    Block,
    PortalIn,
    PortalOut,
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

    fn set_center(&mut self, center:[f32;2]){
        self.x = center[0] - self.width/2.0;
        self.y = center[1] - self.height/2.0;
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

struct Object{
    object_type:ObjectType,
    rect:Rect,
    velocity_x:f32,
    velocity_y:f32,
    gravity:f32,
    ai_direction:f32,
    collision_type:CollisionType,
}

pub struct VectorGraphics {
    objects:Vec<Object>,
    drag:Drag,
    mouse_position:[f32;2],
    mode:Mode,
    speed:f32,
    jump_force:f32,
    left_arrow:bool,
    right_arrow:bool,
    up_arrow:bool,
    last_portal_in:usize,
}

impl VectorGraphics {

    fn is_playing(&self) -> bool{
        match self.mode {
            Mode::Play => true,
            _ => false,
        }
    }

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
            if self.objects[i].rect.contains(point){
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
            speed:5.0,
            jump_force: 14.0,
            left_arrow:false,
            right_arrow:false,
            up_arrow:false,
            last_portal_in:0,
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
                            Some(id) => {
                                self.objects[id].object_type = ObjectType::Player;
                                self.objects[id].gravity = 0.3;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyE=>{
                        match self.find_object_at_point(self.mouse_position){
                            Some(id) => {
                                self.objects[id].object_type = ObjectType::Enemy;
                                self.objects[id].gravity = 0.3;
                                self.objects[id].ai_direction = -1.0;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyI=>{
                        match self.find_object_at_point(self.mouse_position){
                            Some(id) => {
                                self.objects[id].object_type = ObjectType::PortalIn;
                                self.objects[id].collision_type = CollisionType::None;
                                self.last_portal_in = id;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyO => {
                        match self.find_object_at_point(self.mouse_position){
                            Some(id) => {
                                self.objects[id].object_type = ObjectType::PortalOut;
                                self.objects[id].collision_type = CollisionType::None;
                                self.objects[self.last_portal_in].collision_type = CollisionType::PortalTo(id);
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
                    let rect = Rect {
                        x: abs_rect.0, 
                        y: abs_rect.1, 
                        width: abs_rect.2, 
                        height: abs_rect.3,
                    };
                    self.objects.push(Object { 
                        object_type: ObjectType::Block, 
                        rect: rect, 
                        gravity: 0.0,
                        velocity_x: 0.0, 
                        velocity_y: 0.0, 
                        ai_direction: 0.0,
                        collision_type: CollisionType::Bounce,
                    });
                }
            }
            _=>{}
        }
    }

    fn overlaps_any_block(&self, object_id:usize)->Option<usize>{
        for i in 0..self.objects.len(){
            if i!=object_id && Rect::overlaps(&self.objects[object_id].rect, &self.objects[i].rect){
                return Some(i);
            }
        }
        return None;
    }

    fn slide_x(&mut self, id:usize, distance:f32) -> bool{
        self.objects[id].rect.x += distance;
        match self.overlaps_any_block(id) {
            Some(other_id) => {
                match self.objects[other_id].collision_type {
                    CollisionType::Bounce => {
                        self.objects[id].rect.x -= distance;
                        self.objects[id].velocity_x = 0.0;
                        return true;
                    }
                    CollisionType::PortalTo(location_id) => {
                        let location = self.objects[location_id].rect.center();
                        self.objects[id].rect.set_center(location);
                        return false;
                    }
                    _ => return false,
                }
            } 
            None => {}
        }
        false
    }

    fn slide_y(&mut self, id:usize, distance:f32) -> bool{
        self.objects[id].rect.y += distance;
        match self.overlaps_any_block(id) {
            Some(other_id) => {
                match self.objects[other_id].collision_type {
                    CollisionType::Bounce => {
                        self.objects[id].rect.y -= distance;
                        self.objects[id].velocity_y = 0.0;
                        return true;
                    }
                    CollisionType::PortalTo(location_id) => {
                        let location = self.objects[location_id].rect.center();
                        self.objects[id].rect.set_center(location);
                        return false;
                    }
                    _ => return false,
                }
            }
            None => {}
        }
        false
    }

    pub fn update(&mut self, mesh:&mut crate::mesh::Mesh, queue:&wgpu::Queue){
        if self.drag.dragging {
            let abs_rect = Self::abs_rect(self.drag.x, self.drag.y, self.drag.x2 - self.drag.x, self.drag.y2 - self.drag.y);
            mesh.add_rect(abs_rect.0, abs_rect.1, abs_rect.2, abs_rect.3, 0.0, 0.0, 1.0);
        }
        if self.is_playing(){
            for i in 0..self.objects.len(){
                self.objects[i].velocity_y += self.objects[i].gravity;
                self.slide_x(i, self.objects[i].velocity_x);
                let vy = self.objects[i].velocity_y;
                let grounded = self.slide_y(i, vy) && vy >= 0.0;
                match self.objects[i].object_type {
                    ObjectType::Player => {
                        if self.left_arrow{
                            self.slide_x(i, -self.speed);
                        }
                        if self.right_arrow{
                            self.slide_x(i, self.speed);
                        }
                        if self.up_arrow && grounded{
                            self.objects[i].velocity_y -= self.jump_force;
                        }
                    }
                    ObjectType::Enemy => {
                        if self.slide_x(i, self.speed * self.objects[i].ai_direction) {
                            self.objects[i].ai_direction *= -1.0;
                        }
                        if grounded {
                            self.objects[i].velocity_y -= self.jump_force;
                        }
                    }
                    _ => {}
                }
            }
        }
        for object in &self.objects{
            let color = match object.object_type {
                ObjectType::Block => (0.025,0.025,0.025),
                ObjectType::Player => (1.0, 0.5, 0.0),
                ObjectType::Enemy => (1.0, 0.0, 0.0),
                ObjectType::PortalIn => (0.2, 1.0, 0.2),
                ObjectType::PortalOut => (0.2, 0.2, 1.0),
            };
            mesh.add_rect(object.rect.x, object.rect.y, object.rect.width, object.rect.height, color.0, color.1, color.2);
        }
        mesh.update_queue(queue);
    }
}
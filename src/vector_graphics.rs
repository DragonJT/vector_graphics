use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::prelude::*;

struct Drag{
    dragging:bool,
    x:f32,
    y:f32,
    x2:f32,
    y2:f32,
}

#[derive(Serialize, Deserialize)]
enum CollisionType{
    Bounce,
    PortalTo(usize),
    None,
}

enum Mode{
    Play,
    Edit,
}

#[derive(Serialize, Deserialize)]
enum ObjectType{
    Player,
    Enemy,
    Block,
    PortalIn,
    PortalOut,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Vector2{
    pub x:f32,
    pub y:f32,
}

#[derive(Serialize, Deserialize)]
struct Rect{
    x:f32,
    y:f32,
    width:f32,
    height:f32,    
}

impl Rect{
    fn contains(&self, point:Vector2) -> bool{
        point.x > self.x && point.y > self.y && point.x < self.x+self.width && point.y< self.y+self.height
    }

    fn center(&self) -> Vector2{
        Vector2{x:self.x+self.width/2.0, y:self.y+self.height/2.0}
    }

    fn overlaps(a:&Rect, b:&Rect) -> bool{
        let center_a = a.center();
        let center_b = b.center();
        let center_dist_x = (center_a.x - center_b.x).abs();
        let center_dist_y = (center_a.y - center_b.y).abs();
        let radius_dist_x = (a.width + b.width) / 2.0;
        let radius_dist_y = (a.height + b.height)/ 2.0;
        center_dist_x < radius_dist_x && center_dist_y < radius_dist_y
    }
}

#[derive(Serialize, Deserialize)]
struct Object{
    object_type:ObjectType,
    rect:Rect,
    velocity:Vector2,
    gravity:f32,
    ai_direction:f32,
    collision_type:CollisionType,
}

pub struct VectorGraphics {
    objects:Vec<Object>,
    drag:Drag,
    mouse_position:Vector2,
    mode:Mode,
    speed:f32,
    editor_speed:f32,
    jump_force:f32,
    left_arrow:bool,
    right_arrow:bool,
    down_arrow:bool,
    up_arrow:bool,
    last_portal_in:usize,
    cam:Vector2,
    screen:Vector2,
}

impl VectorGraphics {

    fn save(&self){
        let serialized = serde_json::to_string(&self.objects).unwrap();
        let mut file = File::create("save.txt").unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }

    fn load(&self) -> Vec<Object>{
        let mut file = File::open("save.txt").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        serde_json::from_str(&contents).unwrap()
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

    fn find_object_at_point(&self, point:Vector2) -> Option<usize>{
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
            mouse_position:Vector2{x:0.0, y:0.0},
            mode:Mode::Edit,
            speed:5.0,
            editor_speed:7.5,
            jump_force: 14.0,
            left_arrow:false,
            right_arrow:false,
            up_arrow:false,
            down_arrow:false,
            last_portal_in:0,
            cam:Vector2{x:0.0, y:0.0},
            screen:Vector2{x:0.0, y:0.0},
         };
    }

    pub fn mousemove(&mut self, mouse_position:Vector2){
        self.mouse_position = mouse_position;
        let relative_mouse_position = self.get_relative_mouse_position();
        if self.drag.dragging {
            self.drag.x2 = relative_mouse_position.x;
            self.drag.y2 = relative_mouse_position.y;
        }
    }

    fn get_relative_mouse_position(&self) -> Vector2{
        Vector2 { x: self.mouse_position.x + self.cam.x, y: self.mouse_position.y + self.cam.y }
    }

    pub fn resize(&mut self, screen_x:f32, screen_y:f32){
        self.screen.x = screen_x;
        self.screen.y = screen_y;
    }

    pub fn keydown(&mut self, key:winit::keyboard::KeyCode){
        match key {
            winit::keyboard::KeyCode::ArrowLeft=>{ self.left_arrow = true }
            winit::keyboard::KeyCode::ArrowRight=>{ self.right_arrow = true } 
            winit::keyboard::KeyCode::ArrowUp=>{ self.up_arrow = true }
            winit::keyboard::KeyCode::ArrowDown=>{ self.down_arrow = true }
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
                        match self.find_object_at_point(self.get_relative_mouse_position()){
                            Some(id) => {self.objects.remove(id);}
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyR=>{
                        if !self.drag.dragging{
                            let relative_mouse_position = self.get_relative_mouse_position();
                            self.drag.x = relative_mouse_position.x;
                            self.drag.y = relative_mouse_position.y;
                            self.drag.x2 = relative_mouse_position.x;
                            self.drag.y2 = relative_mouse_position.y;
                            self.drag.dragging = true;
                        }
                    }
                    winit::keyboard::KeyCode::KeyP=>{
                        match self.find_object_at_point(self.get_relative_mouse_position()){
                            Some(id) => {
                                self.objects[id].object_type = ObjectType::Player;
                                self.objects[id].gravity = 0.3;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyE=>{
                        match self.find_object_at_point(self.get_relative_mouse_position()){
                            Some(id) => {
                                self.objects[id].object_type = ObjectType::Enemy;
                                self.objects[id].gravity = 0.3;
                                self.objects[id].ai_direction = -1.0;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyI=>{
                        match self.find_object_at_point(self.get_relative_mouse_position()){
                            Some(id) => {
                                self.objects[id].object_type = ObjectType::PortalIn;
                                self.objects[id].collision_type = CollisionType::None;
                                self.last_portal_in = id;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyO => {
                        match self.find_object_at_point(self.get_relative_mouse_position()){
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
                    winit::keyboard::KeyCode::KeyS=>{
                        self.save();
                    }
                    winit::keyboard::KeyCode::KeyL=>{
                        self.objects = self.load();
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
            winit::keyboard::KeyCode::ArrowDown=>{ self.down_arrow = false }
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
                        velocity: Vector2{x:0.0, y:0.0}, 
                        ai_direction: 0.0,
                        collision_type: CollisionType::Bounce,
                    });
                }
            }
            _=>{}
        }
    }

    fn overlaps(&self, object_id:usize)->Vec<usize>{
        let mut result:Vec<usize> = Vec::new();
        for i in 0..self.objects.len(){
            if i!=object_id && Rect::overlaps(&self.objects[object_id].rect, &self.objects[i].rect){
                result.push(i);
            }
        }
        result
    }

    fn portal_to(&mut self, id:usize, location:Vector2) -> bool{
        let old_x = self.objects[id].rect.x;
        let old_y = self.objects[id].rect.y;
        self.objects[id].rect.x = location.x - self.objects[id].rect.width/2.0;
        self.objects[id].rect.y = location.y - self.objects[id].rect.height/2.0;
        for other_id in self.overlaps(id) {
            match self.objects[other_id].collision_type {
                CollisionType::Bounce => {
                    self.objects[id].rect.x = old_x;
                    self.objects[id].rect.y = old_y;
                    return false;
                }
                _ => {}
            }
        }
        true
    }

    fn slide_x(&mut self, id:usize, distance:f32) -> bool{
        self.objects[id].rect.x += distance;
        for other_id in self.overlaps(id) {
            match self.objects[other_id].collision_type {
                CollisionType::Bounce => {
                    self.objects[id].rect.x -= distance;
                    self.objects[id].velocity.x = 0.0;
                    return true;
                }
                CollisionType::PortalTo(location_id) => {
                    let location = self.objects[location_id].rect.center();
                    if self.portal_to(id, location){
                        return false;
                    }
                }
                _ => {},
            }
        }
        false
    }

    fn slide_y(&mut self, id:usize, distance:f32) -> bool{
        self.objects[id].rect.y += distance;
        for other_id in self.overlaps(id) {
            match self.objects[other_id].collision_type {
                CollisionType::Bounce => {
                    self.objects[id].rect.y -= distance;
                    self.objects[id].velocity.y = 0.0;
                    return true;
                }
                CollisionType::PortalTo(location_id) => {
                    let location = self.objects[location_id].rect.center();
                    if self.portal_to(id, location) {
                        return false;
                    }
                }
                _ => {},
            }
        }
        false
    }

    pub fn update(&mut self, mesh:&mut crate::mesh::Mesh, queue:&wgpu::Queue){
        if self.drag.dragging {
            let abs_rect = Self::abs_rect(
                self.drag.x - self.cam.x, 
                self.drag.y - self.cam.y, 
                self.drag.x2 - self.drag.x,
                self.drag.y2 - self.drag.y);
            mesh.add_rect(abs_rect.0, abs_rect.1, abs_rect.2, abs_rect.3, 0.0, 0.0, 1.0);
        }
        match self.mode {
            Mode::Play => {
                for i in 0..self.objects.len(){
                    self.objects[i].velocity.y += self.objects[i].gravity;
                    self.slide_x(i, self.objects[i].velocity.x);
                    let vy = self.objects[i].velocity.y;
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
                                self.objects[i].velocity.y -= self.jump_force;
                            }
                            let player_position = self.objects[i].rect.center();
                            self.cam.x = player_position.x - self.screen.x/2.0;
                            self.cam.y = player_position.y - self.screen.y/2.0;
                        }
                        ObjectType::Enemy => {
                            if self.slide_x(i, self.speed * self.objects[i].ai_direction) {
                                self.objects[i].ai_direction *= -1.0;
                            }
                            if grounded {
                                self.objects[i].velocity.y -= self.jump_force;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Mode::Edit => {
                if self.left_arrow {
                    self.cam.x -= self.editor_speed;
                }
                if self.right_arrow {
                    self.cam.x += self.editor_speed;
                }
                if self.up_arrow {
                    self.cam.y -= self.editor_speed;
                }
                if self.down_arrow {
                    self.cam.y += self.editor_speed;
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
            mesh.add_rect(
                object.rect.x - self.cam.x, 
                object.rect.y - self.cam.y, 
                object.rect.width, 
                object.rect.height, 
                color.0, 
                color.1, 
                color.2);
        }
        mesh.update_queue(queue);
    }
}
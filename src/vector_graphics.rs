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
    PortalTo,
    None,
}

#[derive(Serialize, Deserialize)]
enum Controller{
    None,
    Player,
    AI,
    FollowTarget,
}

enum Mode{
    Play,
    Edit,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct Color{
    r:f32,
    g:f32,
    b:f32,
}


#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Vector2{
    pub x:f32,
    pub y:f32,
}

impl Vector2{
    fn magnitude(&self) -> f32{
        (self.x*self.x + self.y*self.y).sqrt()
    }

    fn normalize(&self) -> Vector2{
        let magnitude = self.magnitude();
        if magnitude == 0.0{
            return Vector2 { x:0.0, y:0.0 };
        }
        Vector2 { x: self.x/magnitude, y: self.y/magnitude }
    }

    fn scale(&self, scale_x:f32, scale_y:f32) -> Vector2{
        Vector2 { x: self.x*scale_x, y: self.y*scale_y }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Rect{
    x:f32,
    y:f32,
    width:f32,
    height:f32,    
}

impl Rect{
    /*fn expand(&self, radius:f32) -> Rect{
        Rect { x: self.x-radius, y: self.y-radius, width: self.width+radius*2.0, height: self.height+radius*2.0 }
    }*/

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

const FACTION_PLAYER:u32 = 1;
const FACTION_ENEMY:u32 = 2;

#[derive(Serialize, Deserialize)]
struct Object{
    faction:u32,
    controller:Controller,
    rect:Rect,
    color:Color,
    velocity:Vector2,
    gravity:f32,
    direction:Vector2,
    collision_type:CollisionType,
    destroying:bool,
    destroy_at_frame:usize,
    enable_firing_at_frame:usize,
    target:usize,
    health:i32,
    max_health:i32,
    damage:i32,
    disable_damage_bar_at_frame:usize,
}

struct Input{
    keys:Vec<winit::keyboard::KeyCode>,
}

impl Input{
    fn new() -> Self{
        Input { keys: Vec::new() }
    }

    fn is_pressed(&self, key:&winit::keyboard::KeyCode) -> bool{
        self.keys.contains(key)
    }

    fn press(&mut self, key:winit::keyboard::KeyCode){
        if !self.keys.contains(&key){
            self.keys.push(key);
        }
    }

    fn release(&mut self, key:&winit::keyboard::KeyCode){
        self.keys.retain(|k| k!=key);
    }
}

pub struct VectorGraphics {
    objects:Vec<Object>,
    drag:Drag,
    mouse_position:Vector2,
    mode:Mode,
    speed:f32,
    editor_speed:f32,
    jump_force:f32,
    last_portal_in:usize,
    cam:Vector2,
    screen:Vector2,
    frame:usize,
    input:Input,
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
            input:Input::new(),
            last_portal_in:0,
            cam:Vector2{x:0.0, y:0.0},
            screen:Vector2{x:0.0, y:0.0},
            frame:0,
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
        self.input.press(key);
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
                            Some(id) => {
                                self.objects[id].destroying = true;
                                self.objects[id].destroy_at_frame = self.frame;
                            }
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
                                self.objects[id].faction = FACTION_PLAYER;
                                self.objects[id].controller = Controller::Player;
                                self.objects[id].color = Color { r:1.0, g:0.5, b:0.0 };
                                self.objects[id].gravity = 0.3;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyE=>{
                        match self.find_object_at_point(self.get_relative_mouse_position()){
                            Some(id) => {
                                self.objects[id].faction = FACTION_ENEMY;
                                self.objects[id].controller = Controller::AI;
                                self.objects[id].color = Color { r:1.0, g:0.0, b:0.0 };
                                self.objects[id].gravity = 0.3;
                                self.objects[id].direction = Vector2{ x:-1.0, y:0.0 };
                                self.objects[id].health = 20;
                                self.objects[id].max_health = 20;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyI=>{
                        match self.find_object_at_point(self.get_relative_mouse_position()){
                            Some(id) => {
                                self.objects[id].color = Color{r:0.2, g:1.0, b:0.2};
                                self.objects[id].collision_type = CollisionType::None;
                                self.last_portal_in = id;
                            }
                            _ => {}
                        }
                    }
                    winit::keyboard::KeyCode::KeyO => {
                        match self.find_object_at_point(self.get_relative_mouse_position()){
                            Some(id) => {
                                self.objects[id].color = Color{r:0.2, g:0.2, b:1.0};
                                self.objects[id].collision_type = CollisionType::None;
                                self.objects[self.last_portal_in].collision_type = CollisionType::PortalTo;
                                self.objects[self.last_portal_in].target = id;
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
        self.input.release(&key);
        match key{
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
                        controller: Controller::None,
                        rect: rect, 
                        color: Color { r: 0.025, g: 0.025, b: 0.025 },
                        gravity: 0.0,
                        velocity: Vector2 { x: 0.0, y: 0.0 }, 
                        direction: Vector2 { x: 0.0, y: 0.0 },
                        collision_type: CollisionType::Bounce,
                        destroying: false,
                        destroy_at_frame: 0,
                        enable_firing_at_frame: 0,
                        target:0,
                        faction:FACTION_PLAYER,
                        health:0,
                        max_health:0,
                        damage:0,
                        disable_damage_bar_at_frame:0,
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

    fn apply_damage_to_id(&mut self, id:usize, other_id:usize){
        if self.objects[id].faction != self.objects[other_id].faction && self.objects[id].max_health>0 && self.objects[other_id].damage != 0{
            self.objects[id].health -= self.objects[other_id].damage;
            self.objects[other_id].damage = 0;
            self.objects[id].disable_damage_bar_at_frame = self.frame+120;
            if self.objects[id].health <= 0{
                self.objects[id].health = 0;
                self.objects[id].destroying = true;
                self.objects[id].destroy_at_frame = self.frame;
            }
        }
    }

    fn apply_damage(&mut self, id:usize, other_id:usize){
        self.apply_damage_to_id(id, other_id);
        self.apply_damage_to_id(other_id, id);
    }

    fn portal_to(&mut self, id:usize, location:Vector2) -> bool{
        let old_x = self.objects[id].rect.x;
        let old_y = self.objects[id].rect.y;
        self.objects[id].rect.x = location.x - self.objects[id].rect.width/2.0;
        self.objects[id].rect.y = location.y - self.objects[id].rect.height/2.0;
        for other_id in self.overlaps(id) {
            self.apply_damage(id, other_id);
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
            self.apply_damage(id, other_id);
            match self.objects[other_id].collision_type {
                CollisionType::Bounce => {
                    self.objects[id].rect.x -= distance;
                    self.objects[id].velocity.x = 0.0;
                    return true;
                }
                CollisionType::PortalTo => {
                    let location = self.objects[self.objects[other_id].target].rect.center();
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
            self.apply_damage(id, other_id);
            match self.objects[other_id].collision_type {
                CollisionType::Bounce => {
                    self.objects[id].rect.y -= distance;
                    self.objects[id].velocity.y = 0.0;
                    return true;
                }
                CollisionType::PortalTo => {
                    let location = self.objects[self.objects[other_id].target].rect.center();
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
                    match self.objects[i].controller {
                        Controller::Player => {
                            if self.input.is_pressed(&winit::keyboard::KeyCode::KeyW){
                                if grounded { 
                                    self.objects[i].velocity.y -= self.jump_force;
                                }
                            }
                            if !(self.input.is_pressed(&winit::keyboard::KeyCode::KeyA) && self.input.is_pressed(&winit::keyboard::KeyCode::KeyD)){
                                if self.input.is_pressed(&winit::keyboard::KeyCode::KeyA){
                                    self.slide_x(i, -self.speed);
                                }
                                if self.input.is_pressed(&winit::keyboard::KeyCode::KeyD){
                                    self.slide_x(i, self.speed);
                                }
                            }
                            
                            if self.input.is_pressed(&winit::keyboard::KeyCode::Space) {
                                if self.objects[i].enable_firing_at_frame <= self.frame{
                                    self.objects[i].enable_firing_at_frame = self.frame+30;
                                    let center = self.objects[i].rect.center();
                                    let mousepos = self.get_relative_mouse_position();
                                    let offset_to_mouse = Vector2 { x:mousepos.x - center.x, y:mousepos.y - center.y };
                                    let direction = offset_to_mouse.normalize().scale(self.objects[i].rect.width, self.objects[i].rect.height);
                                    self.objects.push(Object { 
                                        controller: Controller::FollowTarget, 
                                        rect: self.objects[i].rect.clone(), 
                                        color: Color { r: 1.0, g: 1.0, b: 0.2 }, 
                                        velocity: Vector2 { x: 0.0, y: 0.0 }, 
                                        gravity: 0.0, 
                                        direction, 
                                        collision_type: CollisionType::None, 
                                        destroying: true, 
                                        destroy_at_frame: self.frame+20, 
                                        enable_firing_at_frame: 0,
                                        target: i,
                                        faction: self.objects[i].faction,
                                        health: 0,
                                        max_health: 0,
                                        damage: 5,
                                        disable_damage_bar_at_frame:0,
                                     });
                                }
                            }
                            let player_position = self.objects[i].rect.center();
                            self.cam.x = player_position.x - self.screen.x/2.0;
                            self.cam.y = player_position.y - self.screen.y/2.0;
                        }
                        Controller::AI => {
                            if self.slide_x(i, self.speed * self.objects[i].direction.x) {
                                self.objects[i].direction.x *= -1.0;
                            }
                            if grounded {
                                self.objects[i].velocity.y -= self.jump_force;
                            }
                        }
                        _ => {}
                    }
                    for i in 0..self.objects.len(){
                        match self.objects[i].controller{
                            Controller::FollowTarget => { 
                                let new_center = self.objects[self.objects[i].target].rect.center();
                                self.objects[i].rect.x = new_center.x - self.objects[i].rect.width / 2.0 + self.objects[i].direction.x;
                                self.objects[i].rect.y = new_center.y - self.objects[i].rect.height / 2.0 + self.objects[i].direction.y;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Mode::Edit => {
                if self.input.is_pressed(&winit::keyboard::KeyCode::ArrowLeft) {
                    self.cam.x -= self.editor_speed;
                }
                if self.input.is_pressed(&winit::keyboard::KeyCode::ArrowRight) {
                    self.cam.x += self.editor_speed;
                }
                if self.input.is_pressed(&winit::keyboard::KeyCode::ArrowUp) {
                    self.cam.y -= self.editor_speed;
                }
                if self.input.is_pressed(&winit::keyboard::KeyCode::ArrowDown) {
                    self.cam.y += self.editor_speed;
                }
            }
            
        }
        for object in &self.objects{
            mesh.add_rect(
                object.rect.x - self.cam.x, 
                object.rect.y - self.cam.y, 
                object.rect.width, 
                object.rect.height, 
                object.color.r, 
                object.color.g, 
                object.color.b);
        }
        for object in &self.objects {
            if object.disable_damage_bar_at_frame > self.frame {
                mesh.add_rect(
                    object.rect.x - self.cam.x, 
                    object.rect.y - self.cam.y - object.rect.height/2.0 - 20.0, 
                    object.rect.width * (object.health as f32 / object.max_health as f32), 
                    10.0, 
                    0.0, 
                    1.0, 
                    0.0)
            }
        }

        {
            let mut i = 0;
            while i < self.objects.len(){
                if self.objects[i].destroying && self.objects[i].destroy_at_frame <= self.frame {
                    self.objects.remove(i);
                    for ii in 0..self.objects.len(){
                        if self.objects[ii].target > i{
                            self.objects[ii].target -= 1;
                        }
                    }
                }
                i+=1;
            }
        }
        

        mesh.update_queue(queue);
        self.frame+=1;
    }
}
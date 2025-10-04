use crate::CamTransform;
use cgmath::*;

#[derive(Copy, Clone, Debug)]
pub struct Size2D {
    pub w: u32,
    pub h: u32,
}

impl Size2D {
    pub fn empty() -> Size2D {
        Self { w: 0, h: 0 }
    }
    pub fn from(w: u32, h: u32) -> Self {
        Self { w, h }
    }
    pub(crate) fn shave(&self, n: u32) -> Size2D {
        if self.w > 0 && self.h > 0 {
            Size2D {
                w: self.w - n,
                h: self.h - n,
            }
        } else {
            *self
        }
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h as f32
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Size3D {
    pub w: u32,
    pub h: u32,
    pub d: u32,
}

impl Size3D {
    pub fn empty() -> Size3D {
        Self { w: 0, h: 0, d: 0 }
    }

    pub fn from(w: u32, h: u32, d: u32) -> Self {
        Self { w, h, d }
    }

    pub(crate) fn shave(&self, n: u32) -> Size3D {
        if self.w > 0 && self.h > 0 && self.d > 0 {
            Size3D {
                w: self.w - n,
                h: self.h - n,
                d: self.d - n,
            }
        } else {
            *self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClipDist {
    pub(crate) near: f32,
    pub(crate) far: f32,
}

impl Default for ClipDist {
    fn default() -> Self {
        ClipDist::from(0.01, 1000.0)
    }
}

impl ClipDist {
    pub fn from(near: f32, far: f32) -> ClipDist {
        ClipDist { near, far }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CamProj {
    Ortho,
    Persp,
}

pub struct Camera {
    pub transform: CamTransform,
}

impl Camera {
    pub(crate) fn start(&mut self) {}

    pub(crate) fn pre_update(&mut self) {
        self.transform.calc_matrices();
    }

    pub(crate) fn update(&mut self) {}

    pub(crate) fn post_update(&mut self) {}

    pub(crate) fn end(&mut self) {}
}

impl Camera {
    pub fn new(size: Size2D, proj: CamProj) -> Self {
        let fov = 75.0;
        let clip = ClipDist::default();

        let pos = vec3(0.0, 0.0, 5.0);
        let rot = vec3(0.0, -90.0, 0.0);

        let pos_inverse = Matrix4::from_translation(vec3(-pos.x, -pos.y, -pos.z));
        let rot_inverse = Matrix4::<f32>::from_angle_x(Rad::from(Deg(-rot.x)))
            * Matrix4::<f32>::from_angle_y(Rad::from(Deg(-rot.y)))
            * Matrix4::<f32>::from_angle_z(Rad::from(Deg(-rot.z)));

        let view_matrix = pos_inverse * rot_inverse;

        let mut transform = CamTransform {
            pos,
            rot,
            fov,
            clip,
            size,
            proj,
            view_matrix,
            ortho_scale: 2.0,
            front: vec3(0.0, 0.0, -1.0),
            persp_matrix: Matrix4::identity(),
            ortho_matrix: Matrix4::identity(),
        };
        transform.calc_matrices();

        Camera { transform }
    }

    pub fn fov(&self) -> f32 {
        self.transform.fov
    }
    pub fn ortho_scale(&self) -> f32 {
        self.transform.ortho_scale
    }

    pub fn proj(&self) -> CamProj {
        self.transform.proj
    }

    pub fn clip(&self) -> ClipDist {
        self.transform.clip
    }

    pub fn set_clip(&mut self, clip: ClipDist) {
        self.transform.clip = clip
    }

    pub fn set_clip_near(&mut self, near: f32) {
        self.transform.clip.near = near
    }
    pub fn set_clip_far(&mut self, far: f32) {
        self.transform.clip.far = far
    }
    pub fn set_size(&mut self, size: Size2D) {
        self.transform.size = size;
    }
    pub fn set_proj(&mut self, proj: CamProj) {
        self.transform.proj = proj;
    }

    fn floor_fov(&mut self) {
        if self.transform.fov <= 0.0 {
            self.transform.fov = 0.01;
        }
    }
    pub fn set_fov(&mut self, fov: f32) {
        self.transform.fov = fov;
        self.floor_fov()
    }
    pub fn add_fov(&mut self, value: f32) {
        self.transform.fov += value;
        self.floor_fov()
    }

    pub fn set_ortho_scale(&mut self, value: f32) {
        self.transform.ortho_scale = value;
    }
    pub fn add_ortho_scale(&mut self, value: f32) {
        self.transform.ortho_scale += value;
    }

    pub fn fly_forw(&mut self, speed: f32) {
        self.transform.pos += speed * self.transform.front;
    }
    pub fn fly_back(&mut self, speed: f32) {
        self.transform.pos -= speed * self.transform.front;
    }
    pub fn fly_left(&mut self, speed: f32) {
        self.transform.pos -= speed * self.transform.front.cross(vec3(0.0, 1.0, 0.0).normalize());
    }
    pub fn fly_right(&mut self, speed: f32) {
        self.transform.pos += speed * self.transform.front.cross(vec3(0.0, 1.0, 0.0).normalize());
    }
    pub fn fly_up(&mut self, speed: f32) {
        self.transform.move_y(speed);
    }
    pub fn fly_down(&mut self, speed: f32) {
        self.transform.move_y(-speed);
    }

    pub fn spin_x(&mut self, speed: f32) {
        self.transform.rotate_x(speed)
    }
    pub fn spin_y(&mut self, speed: f32) {
        self.transform.rotate_y(speed)
    }
    pub fn spin_z(&mut self, speed: f32) {
        self.transform.rotate_z(speed)
    }
}

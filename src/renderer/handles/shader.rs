use crate::asset::{bind_image_texture2d_at, bind_texture2d_sampler_at, delete_program};
use crate::renderer::bind_storage_buffer_at;
use crate::{StorageBuffer, Texture2D};
use cgmath::{Matrix, Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};
use gl::types::GLint;
use std::ffi::CString;

pub enum TexSlot {
   S0,
   S1,
   S2,
   S3,
   S4,
   S5,
   S6,
   S7,
   S8,
   S9,
   S10,
   S11,
}

impl TexSlot {
   pub(crate) fn as_index(&self) -> usize {
      match self {
         TexSlot::S0 => 0,
         TexSlot::S1 => 1,
         TexSlot::S2 => 2,
         TexSlot::S3 => 3,
         TexSlot::S4 => 4,
         TexSlot::S5 => 5,
         TexSlot::S6 => 6,
         TexSlot::S7 => 7,
         TexSlot::S8 => 8,
         TexSlot::S9 => 9,
         TexSlot::S10 => 10,
         TexSlot::S11 => 11,
      }
   }
   pub(crate) fn total_slots() -> usize {
      12
   }
}

#[derive(Clone, Debug)]
pub struct Workers {
   pub(crate) group_x: u32,
   pub(crate) group_y: u32,
   pub(crate) group_z: u32,
}

impl Workers {
   pub fn empty() -> Self {
      Self {
         group_x: 0,
         group_y: 0,
         group_z: 0,
      }
   }

   pub fn set_groups(&mut self, x: u32, y: u32, z: u32) {
      self.set_group_x(x);
      self.set_group_y(y);
      self.set_group_z(z);
   }

   pub fn groups(&self) -> (u32, u32, u32) {
      (self.group_x, self.group_y, self.group_z)
   }

   pub fn group_x(&self) -> u32 {
      self.group_x
   }
   pub fn group_y(&self) -> u32 {
      self.group_y
   }
   pub fn group_z(&self) -> u32 {
      self.group_z
   }

   pub fn set_group_x(&mut self, x: u32) {
      self.group_x = x
   }
   pub fn set_group_y(&mut self, y: u32) {
      self.group_y = y
   }
   pub fn set_group_z(&mut self, z: u32) {
      self.group_z = z
   }
}

#[derive(Clone, Debug)]
pub struct Shader {
   pub workers: Workers,
   pub(crate) id: u32,
   pub(crate) is_compute: bool,
   pub(crate) tex_ids: Vec<Option<u32>>,
   pub(crate) sbo_ids: Vec<Option<u32>>,
}

impl Shader {
   pub fn set_tex_at_slot(&mut self, tex: &Texture2D, slot: TexSlot) {
      self.tex_ids[slot.as_index()] = Some(tex.id)
   }
   pub fn set_sbo_at_slot(&mut self, sbo: &StorageBuffer, slot: TexSlot) {
      self.tex_ids[slot.as_index()] = Some(sbo.id)
   }

   pub fn delete(self) {
      delete_program(self.id)
   }

   pub fn bind(&self) {
      unsafe { gl::UseProgram(self.id) }
   }
   pub fn unbind(&self) {
      unsafe { gl::UseProgram(0) }
   }

   pub fn compute(&self) {
      self.bind();
      //CLOSURE FN GO HERE
      //AND HERE
      self.bind_textures();
      let (x, y, z) = self.workers.groups();
      unsafe {
         gl::DispatchCompute(x, y, z);
         gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
      }
   }

   pub fn uniform_location(&self, name: &str) -> Option<u32> {
      unsafe {
         let c_name = CString::new(name).unwrap();
         let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
         if location == -1 {
            None
         } else {
            Some(location as u32)
         }
      }
   }

   pub(crate) fn get_uni_location(&self, name: &str) -> GLint {
      unsafe {
         let c_name = CString::new(name).unwrap();
         let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
         if location == -1 {
            panic!("uniform '{name}' does not exist!");
         } else {
            location
         }
      }
   }

   pub(crate) fn bind_textures(&self) {
      for (slot, tex_id) in self.tex_ids.iter().enumerate() {
         match tex_id {
            None => {}
            Some(id) => match self.is_compute {
               false => bind_texture2d_sampler_at(*id, slot as u32),
               true => bind_image_texture2d_at(*id, slot as u32),
            },
         }
      }
   }
   pub(crate) fn bind_storages(&self) {
      for (slot, sbo_id) in self.sbo_ids.iter().enumerate() {
         match sbo_id {
            None => {}
            Some(id) => bind_storage_buffer_at(*id, slot as u32),
         }
      }
   }

   // ---- scalar ----
   pub(crate) fn set_uni_i32(&self, name: &str, v: i32) {
      unsafe { gl::Uniform1i(self.get_uni_location(name), v) }
   }

   pub(crate) fn set_uni_u32(&self, name: &str, v: u32) {
      unsafe { gl::Uniform1ui(self.get_uni_location(name), v) }
   }

   pub(crate) fn set_uni_f32(&self, name: &str, v: f32) {
      unsafe { gl::Uniform1f(self.get_uni_location(name), v) }
   }

   // ---- vec2 ----
   pub(crate) fn set_uni_vec2_i32(&self, name: &str, v: Vector2<i32>) {
      unsafe { gl::Uniform2i(self.get_uni_location(name), v.x, v.y) }
   }

   pub(crate) fn set_uni_vec2_u32(&self, name: &str, v: Vector2<u32>) {
      unsafe { gl::Uniform2ui(self.get_uni_location(name), v.x, v.y) }
   }

   pub(crate) fn set_uni_vec2_f32(&self, name: &str, v: Vector2<f32>) {
      unsafe { gl::Uniform2f(self.get_uni_location(name), v.x, v.y) }
   }

   // ---- vec3 ----
   pub(crate) fn set_uni_vec3_i32(&self, name: &str, v: Vector3<i32>) {
      unsafe { gl::Uniform3i(self.get_uni_location(name), v.x, v.y, v.z) }
   }

   pub(crate) fn set_uni_vec3_u32(&self, name: &str, v: Vector3<u32>) {
      unsafe { gl::Uniform3ui(self.get_uni_location(name), v.x, v.y, v.z) }
   }

   pub(crate) fn set_uni_vec3_f32(&self, name: &str, v: Vector3<f32>) {
      unsafe { gl::Uniform3f(self.get_uni_location(name), v.x, v.y, v.z) }
   }

   // ---- vec4 ----
   pub(crate) fn set_uni_vec4_i32(&self, name: &str, v: Vector4<i32>) {
      unsafe { gl::Uniform4i(self.get_uni_location(name), v.x, v.y, v.z, v.w) }
   }

   pub(crate) fn set_uni_vec4_u32(&self, name: &str, v: Vector4<u32>) {
      unsafe { gl::Uniform4ui(self.get_uni_location(name), v.x, v.y, v.z, v.w) }
   }

   pub(crate) fn set_uni_vec4_f32(&self, name: &str, v: Vector4<f32>) {
      unsafe { gl::Uniform4f(self.get_uni_location(name), v.x, v.y, v.z, v.w) }
   }

   // ---- matrices ----
   pub(crate) fn set_uni_m2_f32(&self, name: &str, m: Matrix2<f32>) {
      unsafe { gl::UniformMatrix2fv(self.get_uni_location(name), 1, gl::FALSE, m.as_ptr()) }
   }

   pub(crate) fn set_uni_m3_f32(&self, name: &str, m: Matrix3<f32>) {
      unsafe { gl::UniformMatrix3fv(self.get_uni_location(name), 1, gl::FALSE, m.as_ptr()) }
   }

   pub(crate) fn set_uni_m4_f32(&self, name: &str, m: Matrix4<f32>) {
      unsafe { gl::UniformMatrix4fv(self.get_uni_location(name), 1, gl::FALSE, m.as_ptr()) }
   }
}

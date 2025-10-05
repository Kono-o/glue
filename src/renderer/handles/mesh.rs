use crate::asset::ATTRInfo;
use crate::{ATTRType, Transform2D};
use crate::{Shader, Transform3D};
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr};
use std::ffi::c_void;
use std::ptr;

#[derive(Clone, Debug, Copy)]
pub enum DrawMode {
   Points,
   Lines,
   Triangles,
   Strip,
}

impl Default for DrawMode {
   fn default() -> DrawMode {
      DrawMode::Triangles
   }
}

#[derive(Clone, Debug)]
pub(crate) struct MeshHandle {
   pub(crate) layouts: Vec<(ATTRInfo, u32)>,
   pub(crate) draw_mode: DrawMode,
   pub(crate) has_indices: bool,
   pub(crate) vert_count: u32,
   pub(crate) ind_count: u32,
   pub(crate) vao_id: u32,
   pub(crate) buf_id: u32,
   pub(crate) ind_id: u32,
}

macro_rules! mesh_struct {
   ($mesh:ident,$transform:ty) => {
      #[derive(Clone, Debug)]
      pub struct $mesh {
         pub(crate) visibility: bool,
         pub(crate) handle: MeshHandle,
         pub(crate) shader: Option<Shader>,
         pub transform: $transform,
      }

      impl $mesh {
         pub fn set_shader(&mut self, shader: Shader) {
            self.shader = Some(shader)
         }
         pub fn remove_shader(&mut self) {
            self.shader = None
         }
         pub fn get_draw_mode(&self) -> DrawMode {
            self.handle.draw_mode
         }
         pub fn set_draw_mode(&mut self, draw_mode: DrawMode) {
            self.handle.draw_mode = draw_mode
         }

         pub fn index_count(&self) -> u32 {
            self.handle.ind_count
         }
         pub fn vertex_count(&self) -> u32 {
            self.handle.vert_count
         }
         pub fn has_indices(&self) -> bool {
            self.handle.has_indices
         }
         pub fn is_empty(&self) -> bool {
            self.vertex_count() == 0
         }

         pub fn is_visible(&self) -> bool {
            self.visibility || !self.is_empty()
         }

         pub fn set_visibility(&mut self, enable: bool) {
            self.visibility = enable;
         }
         pub fn toggle_visibility(&mut self) {
            self.visibility = !self.visibility;
         }

         pub fn update(&mut self) {
            self.transform.calc_matrix();
         }
      }
   };
}
mesh_struct!(Mesh3D, Transform3D);
mesh_struct!(Mesh2D, Transform2D);

impl Mesh3D {
   pub fn render(&self) {
      if !self.is_visible() {
         return;
      }
      let shader = match &self.shader {
         None => return,
         Some(sh) => sh,
      };
      shader.bind();
      //shader.set_uni_m4_f32("uView", self.cam.transform.view_matrix());
      //shader.set_uni_m4_f32(s, "uProj", self.cam.transform.proj_matrix());

      let tfm = self.transform.matrix();
      shader.set_uni_m4_f32("uTfm", tfm);

      shader.bind_textures();
      shader.bind_storages();
      self.handle.draw()
   }
   pub fn delete(self) {
      self.handle.delete()
   }
}

impl Mesh2D {
   pub fn render(&self) {
      if !self.is_visible() {
         return;
      }
      let shader = match &self.shader {
         None => return,
         Some(sh) => sh,
      };
      shader.bind();

      let _scale = 1.0;
      let _max_layers = 255;
      let tfm = self.transform.matrix();
      let layer = self.transform.layer() as u32;
      //let w = self.cam.transform.size.aspect_ratio() * scale;
      //let proj = ortho(-w, w, -scale, scale, 0.0, -(max_layers + 1) as f32);

      //shader.set_uni_m4_f32("uProj", proj);
      shader.set_uni_m4_f32("uTfm", tfm);
      shader.set_uni_u32("uLayer", layer);

      shader.bind_textures();
      shader.bind_storages();
      self.handle.draw()
   }

   pub fn delete(self) {
      self.handle.delete()
   }
}

impl MeshHandle {
   pub(crate) fn draw(&self) {
      bind_layouts(self.vao_id);
      match self.has_indices {
         false => self.draw_array(),
         true => {
            bind_index_buffer(self.ind_id);
            self.draw_indexed();
         }
      }
   }

   pub(crate) fn draw_indexed(&self) {
      let draw_mode = match_draw_mode(&self.draw_mode);
      unsafe {
         gl::DrawElements(
            draw_mode,
            self.ind_count as GLsizei,
            gl::UNSIGNED_INT,
            ptr::null(),
         );
      }
   }

   pub(crate) fn draw_array(&self) {
      let draw_mode = match_draw_mode(&self.draw_mode);
      unsafe {
         gl::DrawArrays(draw_mode, 0, self.vert_count as GLsizei);
      }
   }

   pub(crate) fn delete(self) {
      delete_mesh_buffer(self.vao_id, self.buf_id);
      delete_index_buffer(self.ind_id);
   }
}

fn match_draw_mode(dm: &DrawMode) -> GLenum {
   match dm {
      DrawMode::Points => gl::POINTS,
      DrawMode::Lines => gl::LINES,
      DrawMode::Triangles => gl::TRIANGLES,
      DrawMode::Strip => gl::TRIANGLE_STRIP,
   }
}

//BUFFERS
pub(crate) fn create_mesh_buffer() -> (u32, u32) {
   let (mut v_id, mut b_id): (u32, u32) = (0, 0);
   unsafe {
      gl::GenVertexArrays(1, &mut v_id);
      gl::GenBuffers(1, &mut b_id);
   }
   (v_id, b_id)
}

pub(crate) fn delete_mesh_buffer(v_id: u32, b_id: u32) {
   unsafe {
      gl::DeleteVertexArrays(1, &v_id);
      gl::DeleteBuffers(1, &b_id);
   }
}

//VAO
pub(crate) fn bind_layouts(v_id: u32) {
   unsafe {
      gl::BindVertexArray(v_id);
   }
}

pub(crate) fn set_attr_layout(attr: &ATTRInfo, attr_id: u32, stride: usize, local_offset: usize) {
   unsafe {
      gl::VertexAttribPointer(
         attr_id,
         attr.elem_count as GLint,
         match_attr_type(&attr.typ),
         gl::FALSE,
         stride as GLsizei,
         match local_offset {
            0 => ptr::null(),
            _ => local_offset as *const c_void,
         },
      );
      gl::EnableVertexAttribArray(attr_id);
   }
}

pub(crate) fn unbind_layouts() {
   unsafe {
      gl::BindVertexArray(0);
   }
}

//VBO
pub(crate) fn bind_buffer(id: u32) {
   unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, id);
   }
}

pub(crate) fn fill_buffer(id: u32, data: &[u8]) {
   unsafe {
      bind_buffer(id);

      gl::BufferData(
         gl::ARRAY_BUFFER,
         data.len() as GLsizeiptr,
         &data[0] as *const u8 as *const c_void,
         gl::DYNAMIC_DRAW,
      );
   }
}

pub(crate) fn subfill_buffer(id: u32, offset: usize, data: &[u8]) {
   unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, id);
      gl::BufferSubData(
         gl::ARRAY_BUFFER,
         offset as isize,
         data.len() as isize,
         data.as_ptr() as *const c_void,
      );
   }
}

pub(crate) fn resize_buffer(id: u32, size: usize) {
   unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, id);
      gl::BufferData(
         gl::ARRAY_BUFFER,
         size as GLsizeiptr,
         std::ptr::null(),
         gl::DYNAMIC_DRAW,
      );
   }
}

pub(crate) fn unbind_buffer() {
   unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
   }
}

//EBO
pub(crate) fn create_index_buffer() -> u32 {
   let mut id: u32 = 0;
   unsafe {
      gl::GenBuffers(1, &mut id);
   }
   id
}

pub(crate) fn bind_index_buffer(id: u32) {
   unsafe {
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
   }
}

pub(crate) fn fill_index_buffer(id: u32, data: &[u32]) {
   unsafe {
      bind_index_buffer(id);
      gl::BufferData(
         gl::ELEMENT_ARRAY_BUFFER,
         (data.len() * size_of::<GLint>()) as GLsizeiptr,
         &data[0] as *const u32 as *const c_void,
         gl::DYNAMIC_DRAW,
      );
   }
}

pub(crate) fn unbind_index_buffer() {
   unsafe {
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
   }
}

pub(crate) fn delete_index_buffer(id: u32) {
   unsafe {
      gl::DeleteBuffers(1, &id);
   }
}

//SBO
pub enum SBOSlot {
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

impl SBOSlot {
   pub(crate) fn as_index(&self) -> usize {
      match self {
         SBOSlot::S0 => 0,
         SBOSlot::S1 => 1,
         SBOSlot::S2 => 2,
         SBOSlot::S3 => 3,
         SBOSlot::S4 => 4,
         SBOSlot::S5 => 5,
         SBOSlot::S6 => 6,
         SBOSlot::S7 => 7,
         SBOSlot::S8 => 8,
         SBOSlot::S9 => 9,
         SBOSlot::S10 => 10,
         SBOSlot::S11 => 11,
      }
   }
   pub(crate) fn total_slots() -> usize {
      12
   }
}

pub struct StorageBuffer {
   pub(crate) id: u32,
   pub(crate) size: usize,
}

impl StorageBuffer {
   pub(crate) fn bind(&self) {
      bind_storage_buffer(self.id);
   }

   pub fn new(size: usize) -> StorageBuffer {
      let id = create_storage_buffer();
      resize_storage_buffer(id, size);
      StorageBuffer { id, size }
   }
   pub fn resize(&mut self, size: usize) {
      self.bind();
      if size != self.size {
         self.size = size;
         resize_storage_buffer(self.id, self.size);
      }
   }

   pub fn fill(&mut self, data: &[u8]) {
      self.bind();
      let len = data.len();
      self.resize(len);
      fill_storage_buffer(self.id, data)
   }
   pub fn subfill(&mut self, offset: usize, data: &[u8]) {
      self.bind();
      let len = data.len() + offset;
      self.resize(len);
      subfill_storage_buffer(self.id, offset, data)
   }
   pub fn fetch(&self) -> Vec<u8> {
      self.bind();
      read_storage_buffer(self.id, self.size)
   }
   pub fn delete(self) {
      delete_storage_buffer(self.id);
      unbind_storage_buffer()
   }
}

pub(crate) fn create_storage_buffer() -> u32 {
   let mut id: u32 = 0;
   unsafe {
      gl::GenBuffers(1, &mut id);
   }
   id
}

pub(crate) fn bind_storage_buffer(id: u32) {
   unsafe {
      gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
   }
}

pub(crate) fn bind_storage_buffer_at(id: u32, slot: u32) {
   unsafe {
      gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, slot, id);
   }
}

pub(crate) fn fill_storage_buffer(id: u32, buffer: &[u8]) {
   unsafe {
      bind_storage_buffer(id);
      gl::BufferData(
         gl::SHADER_STORAGE_BUFFER,
         buffer.len() as GLsizeiptr,
         buffer.as_ptr() as *const c_void,
         gl::DYNAMIC_DRAW,
      );
   }
}

pub(crate) fn subfill_storage_buffer(id: u32, offset: usize, data: &[u8]) {
   unsafe {
      gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
      gl::BufferSubData(
         gl::SHADER_STORAGE_BUFFER,
         offset as isize,
         data.len() as isize,
         data.as_ptr() as *const c_void,
      );
   }
}

pub(crate) fn resize_storage_buffer(id: u32, size: usize) {
   unsafe {
      bind_storage_buffer(id);
      gl::BufferData(
         gl::SHADER_STORAGE_BUFFER,
         size as GLsizeiptr,
         ptr::null(),
         gl::DYNAMIC_DRAW,
      );
   }
}

pub(crate) fn read_storage_buffer(id: u32, size: usize) -> Vec<u8> {
   unsafe {
      bind_storage_buffer(id);
      let mut data = vec![0u8; size];
      gl::GetBufferSubData(
         gl::SHADER_STORAGE_BUFFER,
         0,
         size as GLsizeiptr,
         data.as_mut_ptr() as *mut c_void,
      );
      data
   }
}

pub(crate) fn unbind_storage_buffer() {
   unsafe {
      gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
   }
}

pub(crate) fn delete_storage_buffer(id: u32) {
   unsafe {
      gl::DeleteBuffers(1, &id);
   }
}

fn match_attr_type(attr_type: &ATTRType) -> GLenum {
   match attr_type {
      ATTRType::I8 => gl::BYTE,
      ATTRType::U8 => gl::UNSIGNED_BYTE,
      ATTRType::I16 => gl::SHORT,
      ATTRType::U16 => gl::UNSIGNED_SHORT,
      ATTRType::I32 => gl::INT,
      ATTRType::U32 => gl::UNSIGNED_INT,
      ATTRType::F32 => gl::FLOAT,
      ATTRType::F64 => gl::DOUBLE,
   }
}

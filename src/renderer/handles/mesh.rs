use crate::Transform2D;
use crate::asset::ATTRInfo;
use crate::{Shader, Transform3D};

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
         pub(crate) draw_mode: DrawMode,
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
            self.draw_mode
         }
         pub fn set_draw_mode(&mut self, draw_mode: DrawMode) {
            self.draw_mode = draw_mode
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

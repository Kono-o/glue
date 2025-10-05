use crate::asset::util;
use crate::*;
use cgmath::Vector2;
use std::collections::HashMap;
use std::ops::Deref;

enum OBJ {
   Parsed {
      pos_attr: Pos3DATTR,
      col_attr: ColATTR,
      uvm_attr: UVMATTR,
      nrm_attr: NrmATTR,
      ind_attr: IndATTR,
   },
   NonTriangle(String),
}
impl OBJ {
   fn parse(src: &str) -> OBJ {
      let mut pos_attr = Pos3DATTR::empty();
      let mut col_attr = ColATTR::empty();
      let mut uvm_attr = UVMATTR::empty();
      let mut nrm_attr = NrmATTR::empty();
      let mut ind_attr = IndATTR::empty();

      let mut pos_data = Vec::new();
      let mut uvm_data = Vec::new();
      let mut nrm_data = Vec::new();
      type Vert = Vec<usize>;
      let mut verts: Vec<Vert> = Vec::new();
      let mut unique_verts = HashMap::new();

      for line in src.lines() {
         let line = line.trim();
         let words = line.split(' ').collect::<Vec<&str>>();
         if words.is_empty() {
            continue;
         }
         match words[0] {
            "v" => pos_data.push(words.parse_3_to_f32()),
            "vt" => uvm_data.push(words.parse_2_to_f32()),
            "vn" => nrm_data.push(words.parse_3_to_f32()),
            "f" => {
               if words.len() != 4 {
                  return OBJ::NonTriangle(line.to_string());
               }
               for word in &words[1..] {
                  let tokens = word.split('/').collect::<Vec<&str>>();
                  let vert = tokens.parse_to_usize();
                  verts.push(vert);
               }
            }
            _ => {}
         }
      }
      let attr_count = verts[0].len();
      let pos_exists = attr_count > 0;
      let uvm_exists = attr_count > 1;
      let nrm_exists = attr_count > 2;

      let def_uvm = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]];
      let def_col = [1.0, 1.0, 1.0, 1.0];
      let def_nrm = [1.0, 1.0, 1.0];
      for (i, vert) in verts.iter().enumerate() {
         let pos_index = match pos_exists {
            true => Some(vert[0]),
            _ => None,
         };
         let uvm_index = match uvm_exists {
            true => Some(vert[1]),
            _ => None,
         };
         let nrm_index = match nrm_exists {
            true => Some(vert[2]),
            _ => None,
         };

         let key = (pos_index, uvm_index, nrm_index);
         if unique_verts.contains_key(&key) {
            let idx = unique_verts[&key] as u32;
            ind_attr.push(idx);
         } else {
            let v_local = i % 3;
            let new = pos_attr.data.len();
            unique_verts.insert(key, new);
            pos_attr.push(match pos_index {
               Some(id) => pos_data[id],
               None => [0.0; 3],
            });
            uvm_attr.push(match uvm_index {
               Some(id) => uvm_data[id],
               None => def_uvm[v_local],
            });
            nrm_attr.push(match nrm_index {
               Some(id) => nrm_data[id],
               None => def_nrm,
            });
            col_attr.push(def_col);
            ind_attr.push(new as u32);
         }
      }
      OBJ::Parsed {
         pos_attr,
         col_attr,
         uvm_attr,
         nrm_attr,
         ind_attr,
      }
   }
}

#[derive(Debug)]
pub struct Mesh3DFile {
   pub(crate) pos_attr: Pos3DATTR,
   pub(crate) col_attr: ColATTR,
   pub(crate) uvm_attr: UVMATTR,
   pub(crate) nrm_attr: NrmATTR,
   pub(crate) ind_attr: IndATTR,
   pub(crate) cus_attrs: Vec<CustomATTR>,
}

impl Mesh3DFile {
   pub fn empty() -> Mesh3DFile {
      Mesh3DFile {
         pos_attr: Pos3DATTR::empty(),
         col_attr: ColATTR::empty(),
         uvm_attr: UVMATTR::empty(),
         nrm_attr: NrmATTR::empty(),
         ind_attr: IndATTR::empty(),
         cus_attrs: Vec::new(),
      }
   }

   pub fn set_pos_attr(&mut self, pos_attr: Pos3DATTR) {
      self.pos_attr = pos_attr
   }
   pub fn set_col_attr(&mut self, col_attr: ColATTR) {
      self.col_attr = col_attr;
   }

   pub fn set_uvm_attr(&mut self, uvm_attr: UVMATTR) {
      self.uvm_attr = uvm_attr;
   }

   pub fn set_nrm_attr(&mut self, nrm_attr: NrmATTR) {
      self.nrm_attr = nrm_attr;
   }

   pub fn set_ind_attr(&mut self, ind_attr: IndATTR) {
      self.ind_attr = ind_attr;
   }

   fn from_path(path: &str) -> Result<Mesh3DFile, GLueError> {
      let wierd = Err(GLueError::from(GLueErrorKind::WierdFile, path));
      match file::name(path) {
         None => return wierd,
         Some(n) => n,
      };
      match file::ex(path) {
         None => return wierd,
         Some(ex) => match ex.eq_ignore_ascii_case(util::ex::OBJ) {
            false => return wierd,
            true => ex,
         },
      };

      if file::exists_on_disk(path) {
         let obj_src = match file::read_as_string(path) {
            Err(e) => return Err(e),
            Ok(o_src) => o_src,
         };
         let msh = match OBJ::parse(&obj_src) {
            OBJ::NonTriangle(line) => {
               return Err(GLueError::from(
                  GLueErrorKind::NotTriangle,
                  &format!("{path} -> line {line}"),
               ));
            }
            OBJ::Parsed {
               pos_attr,
               col_attr,
               uvm_attr,
               nrm_attr,
               ind_attr,
            } => Mesh3DFile {
               cus_attrs: Vec::new(),
               pos_attr,
               col_attr,
               uvm_attr,
               nrm_attr,
               ind_attr,
            },
         };
         Ok(msh)
      } else {
         Err(GLueError::from(
            GLueErrorKind::Missing,
            &format!("file missing {path}"),
         ))
      }
   }

   pub fn attach_custom_attr(&mut self, cus_attr: CustomATTR) {
      self.cus_attrs.push(cus_attr);
   }

   pub fn has_no_attr(&self) -> bool {
      let no_attr = self.starts_with_custom();
      let no_cus_attr = self.cus_attrs.len() == 0;
      no_attr && no_cus_attr
   }

   pub fn starts_with_custom(&self) -> bool {
      self.pos_attr.is_empty()
         && self.col_attr.is_empty()
         && self.uvm_attr.is_empty()
         && self.nrm_attr.is_empty()
   }

   pub(crate) fn has_custom_attrs(&self) -> bool {
      !self.cus_attrs.is_empty()
   }

   pub fn ship(self) -> Mesh3D {
      let handle = create_mesh3d_handle(&self);
      Mesh3D {
         handle,
         visibility: true,
         shader: None,
         transform: Transform3D::default(),
      }
   }
}

trait ParseWords {
   fn parse_2_to_f32(&self) -> [f32; 2];
   fn parse_3_to_f32(&self) -> [f32; 3];
   fn parse_to_usize(&self) -> Vec<usize>;
}
impl ParseWords for Vec<&str> {
   fn parse_2_to_f32(&self) -> [f32; 2] {
      const N: usize = 2;
      let mut elem = [0.0; N];
      for i in 1..=N {
         elem[i - 1] = self[i].parse::<f32>().unwrap_or(0.0)
      }
      elem[1] = 1.0 - elem[1];
      elem
   }
   fn parse_3_to_f32(&self) -> [f32; 3] {
      const N: usize = 3;
      let mut elem = [0.0; N];
      for i in 1..=N {
         elem[i - 1] = self[i].parse::<f32>().unwrap_or(0.0)
      }
      elem
   }
   fn parse_to_usize(&self) -> Vec<usize> {
      let mut elem: Vec<usize> = Vec::new();
      for str in self {
         elem.push(str.parse::<usize>().unwrap_or(1) - 1);
      }
      elem
   }
}

#[derive(Debug)]
pub struct Mesh2DFile {
   pub(crate) pos_attr: Pos2DATTR,
   pub(crate) layer: u8,
   pub(crate) aspect: f32,
   pub(crate) col_attr: ColATTR,
   pub(crate) uvm_attr: UVMATTR,
   pub(crate) ind_attr: IndATTR,
   pub(crate) cus_attrs: Vec<CustomATTR>,
}

pub enum Center {
   TopLeft,
   TopRight,
   BottomLeft,
   BottomRight,
   Middle,
   Custom(f32, f32),
}

impl Center {
   pub(crate) fn offset(&self) -> Vector2<f32> {
      let x = 1.0;
      let y = 1.0;
      let vec = match self {
         Center::TopLeft => Vector2::new(x, -y),
         Center::TopRight => Vector2::new(-x, -y),
         Center::BottomRight => Vector2::new(-x, y),
         Center::BottomLeft => Vector2::new(x, y),
         Center::Middle => Vector2::new(0.0, 0.0),
         Center::Custom(x, y) => Vector2::new(-x, -y),
      };
      vec
   }
}
impl Mesh2DFile {
   pub fn empty() -> Mesh2DFile {
      Mesh2DFile {
         pos_attr: Pos2DATTR::empty(),
         layer: 0,
         aspect: 1.0,
         col_attr: ColATTR::empty(),
         uvm_attr: UVMATTR::empty(),
         ind_attr: IndATTR::empty(),
         cus_attrs: Vec::new(),
      }
   }
   pub(crate) fn offset_pos_by_center(&mut self, center: &Center) {
      let offset = center.offset();
      for mut pos in self.pos_attr.data.iter_mut() {
         pos[0] += offset.x * self.aspect;
         pos[1] += offset.y;
      }
   }

   pub fn set_pos_attr(&mut self, pos_attr: Pos2DATTR) {
      self.pos_attr = pos_attr
   }

   pub fn set_layer(&mut self, layer: u8) {
      self.layer = layer
   }

   pub fn set_center(&mut self, center: Center) {
      self.offset_pos_by_center(&center);
   }

   pub fn set_col_attr(&mut self, col_attr: ColATTR) {
      self.col_attr = col_attr;
   }

   pub fn set_uvm_attr(&mut self, uvm_attr: UVMATTR) {
      self.uvm_attr = uvm_attr;
   }

   pub fn set_ind_attr(&mut self, ind_attr: IndATTR) {
      self.ind_attr = ind_attr;
   }

   pub fn quad(size: &Size2D) -> Mesh2DFile {
      let mut mesh = Mesh2DFile::empty();

      mesh.aspect = size.aspect_ratio();
      let x = mesh.aspect;
      let y = 1.0;
      let pos_attr = Pos2DATTR::from_array(&[[-x, y], [x, y], [x, -y], [-x, -y]]);
      mesh.set_pos_attr(pos_attr);
      mesh.offset_pos_by_center(&Center::Middle);

      let col_attr = ColATTR::from_array(&[[1.0, 1.0, 1.0, 1.0]; 4]);
      let uvm_attr = UVMATTR::from_array(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
      let ind_attr = IndATTR::from_array(&[0, 2, 1, 2, 0, 3]);

      mesh.set_col_attr(col_attr);
      mesh.set_uvm_attr(uvm_attr);
      mesh.set_ind_attr(ind_attr);
      mesh
   }

   pub fn attach_custom_attr(&mut self, cus_attr: CustomATTR) {
      self.cus_attrs.push(cus_attr);
   }

   pub fn has_no_attr(&self) -> bool {
      let no_attr = self.starts_with_custom();
      let no_cus_attr = self.cus_attrs.len() == 0;
      no_attr && no_cus_attr
   }

   pub fn starts_with_custom(&self) -> bool {
      self.pos_attr.is_empty() && self.col_attr.is_empty() && self.uvm_attr.is_empty()
   }

   pub(crate) fn has_custom_attrs(&self) -> bool {
      !self.cus_attrs.is_empty()
   }

   pub fn ship(self) -> Mesh2D {
      let handle = create_mesh2d_handle(&self);
      Mesh2D {
         handle,
         visibility: true,
         shader: None,
         transform: Transform2D::default(),
      }
   }
}

fn create_mesh3d_handle(mesh: &Mesh3DFile) -> MeshHandle {
   let (vao_id, buf_id) = create_mesh_buffer();
   let ind_id = create_index_buffer();

   let (mut pos_info, mut pos_data) = (&ATTRInfo::empty(), &Vec::new());
   let (mut col_info, mut col_data) = (&ATTRInfo::empty(), &Vec::new());
   let (mut uvm_info, mut uvm_data) = (&ATTRInfo::empty(), &Vec::new());
   let (mut nrm_info, mut nrm_data) = (&ATTRInfo::empty(), &Vec::new());
   let (mut ind_info, mut ind_data) = (&ATTRInfo::empty(), &Vec::new());

   let mut cus_infos: Vec<&ATTRInfo> = Vec::new();
   let mut cus_datas: Vec<&Vec<u8>> = Vec::new();

   let (mut ind_count, mut vert_count, mut stride) = (0, 0, 0);

   let mut pos_exists = !mesh.pos_attr.is_empty();
   let mut col_exists = !mesh.col_attr.is_empty();
   let mut uvm_exists = !mesh.uvm_attr.is_empty();
   let mut nrm_exists = !mesh.nrm_attr.is_empty();

   if pos_exists {
      pos_info = &mesh.pos_attr.info;
      pos_data = &mesh.pos_attr.data;
      stride += pos_info.elem_count * pos_info.byte_count;
   }
   if col_exists {
      col_info = &mesh.col_attr.info;
      col_data = &mesh.col_attr.data;
      stride += col_info.elem_count * col_info.byte_count;
   }
   if uvm_exists {
      uvm_info = &mesh.uvm_attr.info;
      uvm_data = &mesh.uvm_attr.data;
      stride += uvm_info.elem_count * uvm_info.byte_count;
   }
   if nrm_exists {
      nrm_info = &mesh.nrm_attr.info;
      nrm_data = &mesh.nrm_attr.data;
      stride += nrm_info.elem_count * nrm_info.byte_count;
   }
   for cus_attr in mesh.cus_attrs.iter() {
      let cus_info = &cus_attr.info;
      let cus_data = &cus_attr.data;
      stride += cus_info.elem_count * cus_info.byte_count;
      cus_infos.push(cus_info);
      cus_datas.push(cus_data);
   }
   let mut end = pos_data.len();
   if mesh.starts_with_custom() {
      end = cus_datas[0].len() / (cus_infos[0].byte_count * cus_infos[0].elem_count);
   }

   let mut buffer: Vec<u8> = Vec::new();
   for i in 0..end {
      vert_count += 1;
      if pos_exists {
         buffer.push_attr(&pos_data[i]);
      }
      if col_exists {
         buffer.push_attr(&col_data[i]);
      }
      if uvm_exists {
         buffer.push_attr(&uvm_data[i]);
      }
      if nrm_exists {
         buffer.push_attr(&nrm_data[i]);
      }

      for (j, _attr) in mesh.cus_attrs.iter().enumerate() {
         let cus_byte_count = cus_infos[j].byte_count * cus_infos[j].elem_count;
         let cus_data = cus_datas[j];
         let start = i * cus_byte_count;
         let end = ((i + 1) * (cus_byte_count)) - 1;
         buffer.push_attr(&cus_data[start..=end]);
      }
   }

   let mut attr_id = 0;
   let mut local_offset = 0;

   bind_layouts(vao_id);
   bind_buffer(buf_id);

   let mut layouts: Vec<(ATTRInfo, u32)> = Vec::new();
   if pos_exists {
      set_attr_layout(&pos_info, attr_id, stride, local_offset);
      local_offset += pos_info.elem_count * pos_info.byte_count;
      layouts.push((pos_info.clone(), attr_id));
      attr_id += 1;
   }
   if col_exists {
      set_attr_layout(&col_info, attr_id, stride, local_offset);
      local_offset += col_info.elem_count * col_info.byte_count;
      layouts.push((col_info.clone(), attr_id));
      attr_id += 1;
   }
   if uvm_exists {
      set_attr_layout(&uvm_info, attr_id, stride, local_offset);
      local_offset += uvm_info.elem_count * uvm_info.byte_count;
      layouts.push((uvm_info.clone(), attr_id));
      attr_id += 1;
   }
   if nrm_exists {
      set_attr_layout(&nrm_info, attr_id, stride, local_offset);
      local_offset += nrm_info.elem_count * nrm_info.byte_count;
      layouts.push((nrm_info.clone(), attr_id));
      attr_id += 1;
   }

   for cus_info in cus_infos.iter() {
      set_attr_layout(cus_info, attr_id, stride, local_offset);
      local_offset += cus_info.elem_count * cus_info.byte_count;
      layouts.push((cus_info.deref().clone(), attr_id));
      attr_id += 1;
   }

   if buffer.len() > 0 {
      fill_buffer(buf_id, &buffer);
   }
   unbind_buffer();

   let mut has_indices = false;
   let mut index_buffer: Vec<u32> = Vec::new();

   if !mesh.ind_attr.is_empty() {
      ind_info = &mesh.ind_attr.info;
      ind_data = &mesh.ind_attr.data;
      has_indices = true;
      for index in ind_data.iter() {
         ind_count += 1;
         index_buffer.push(*index);
      }
      bind_index_buffer(ind_id);
      fill_index_buffer(ind_id, &index_buffer);
      unbind_index_buffer();
   }
   let draw_mode = DrawMode::default();
   MeshHandle {
      layouts,
      draw_mode,
      has_indices,
      vert_count,
      ind_count,
      vao_id,
      buf_id,
      ind_id,
   }
}
fn create_mesh2d_handle(mesh: &Mesh2DFile) -> MeshHandle {
   let (vao_id, buf_id) = create_mesh_buffer();
   let ind_id = create_index_buffer();

   let (mut pos_info, mut pos_data) = (&ATTRInfo::empty(), &Vec::new());
   let (mut col_info, mut col_data) = (&ATTRInfo::empty(), &Vec::new());
   let (mut uvm_info, mut uvm_data) = (&ATTRInfo::empty(), &Vec::new());
   let (mut ind_info, mut ind_data) = (&ATTRInfo::empty(), &Vec::new());

   let mut cus_infos: Vec<&ATTRInfo> = Vec::new();
   let mut cus_datas: Vec<&Vec<u8>> = Vec::new();

   let (mut ind_count, mut vert_count, mut stride) = (0, 0, 0);

   let mut pos_exists = !mesh.pos_attr.is_empty();
   let mut col_exists = !mesh.col_attr.is_empty();
   let mut uvm_exists = !mesh.uvm_attr.is_empty();

   if pos_exists {
      pos_info = &mesh.pos_attr.info;
      pos_data = &mesh.pos_attr.data;
      stride += pos_info.elem_count * pos_info.byte_count;
   }
   if col_exists {
      col_info = &mesh.col_attr.info;
      col_data = &mesh.col_attr.data;
      stride += col_info.elem_count * col_info.byte_count;
   }
   if uvm_exists {
      uvm_info = &mesh.uvm_attr.info;
      uvm_data = &mesh.uvm_attr.data;
      stride += uvm_info.elem_count * uvm_info.byte_count;
   }

   for cus_attr in mesh.cus_attrs.iter() {
      let cus_info = &cus_attr.info;
      let cus_data = &cus_attr.data;
      stride += cus_info.elem_count * cus_info.byte_count;
      cus_infos.push(cus_info);
      cus_datas.push(cus_data);
   }

   let mut end = pos_data.len();
   if mesh.starts_with_custom() {
      end = cus_datas[0].len() / (cus_infos[0].byte_count * cus_infos[0].elem_count);
   }

   let mut buffer: Vec<u8> = Vec::new();
   for i in 0..end {
      vert_count += 1;
      if pos_exists {
         buffer.push_attr(&pos_data[i]);
      }
      if col_exists {
         buffer.push_attr(&col_data[i]);
      }
      if uvm_exists {
         buffer.push_attr(&uvm_data[i]);
      }

      for (j, _attr) in mesh.cus_attrs.iter().enumerate() {
         let cus_byte_count = cus_infos[j].byte_count * cus_infos[j].elem_count;
         let cus_data = cus_datas[j];
         let start = i * cus_byte_count;
         let end = ((i + 1) * (cus_byte_count)) - 1;
         buffer.push_attr(&cus_data[start..=end]);
      }
   }

   let mut attr_id = 0;
   let mut local_offset = 0;

   bind_layouts(vao_id);
   bind_buffer(buf_id);

   let mut layouts: Vec<(ATTRInfo, u32)> = Vec::new();
   if pos_exists {
      set_attr_layout(&pos_info, attr_id, stride, local_offset);
      local_offset += pos_info.elem_count * pos_info.byte_count;
      layouts.push((pos_info.clone(), attr_id));
      attr_id += 1;
   }
   if col_exists {
      set_attr_layout(&col_info, attr_id, stride, local_offset);
      local_offset += col_info.elem_count * col_info.byte_count;
      layouts.push((col_info.clone(), attr_id));
      attr_id += 1;
   }
   if uvm_exists {
      set_attr_layout(&uvm_info, attr_id, stride, local_offset);
      local_offset += uvm_info.elem_count * uvm_info.byte_count;
      layouts.push((uvm_info.clone(), attr_id));
      attr_id += 1;
   }

   for cus_info in cus_infos.iter() {
      set_attr_layout(cus_info, attr_id, stride, local_offset);
      local_offset += cus_info.elem_count * cus_info.byte_count;
      layouts.push((cus_info.deref().clone(), attr_id));
      attr_id += 1;
   }

   if buffer.len() > 0 {
      fill_buffer(buf_id, &buffer);
   }
   unbind_buffer();

   let mut has_indices = false;
   let mut index_buffer: Vec<u32> = Vec::new();

   if !mesh.ind_attr.is_empty() {
      ind_info = &mesh.ind_attr.info;
      ind_data = &mesh.ind_attr.data;
      has_indices = true;
      for index in ind_data.iter() {
         ind_count += 1;
         index_buffer.push(*index);
      }
      bind_index_buffer(ind_id);
      fill_index_buffer(ind_id, &index_buffer);
      unbind_index_buffer();
   }
   let draw_mode = DrawMode::default();
   MeshHandle {
      layouts,
      draw_mode,
      has_indices,
      vert_count,
      ind_count,
      vao_id,
      buf_id,
      ind_id,
   }
}

trait Buffer {
   fn push_attr<T: DataType>(&mut self, attr: &[T]);
}

impl Buffer for Vec<u8> {
   fn push_attr<T: DataType>(&mut self, attr: &[T]) {
      for elem in attr.iter() {
         let bytes = elem.u8ify();
         for byte in bytes.iter() {
            self.push(*byte);
         }
      }
   }
}

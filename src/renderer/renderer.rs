use crate::asset::ATTRInfo;
use crate::renderer::gl::{GLError, GL};
use crate::renderer::handles::{DrawMode, NEMesh3D, Shader};
use crate::renderer::MeshHandle;
use crate::{
    CamProj, Camera, DataType, Mesh2DFile, Mesh3DFile, NEMesh2D, ShaderFile, Size2D, Texture, TextureFile,
    Transform2D, Transform3D, RGBA,
};
use cgmath::ortho;
use std::ops::Deref;

#[derive(Copy, Clone)]
pub enum PolyMode {
    Points,
    WireFrame,
    Filled,
}
#[derive(Copy, Clone)]
pub enum Cull {
    Clock,
    AntiClock,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ShaderType {
    Vert,
    Frag,
}

pub enum RendError {
    GLError(GLError),
}

pub struct Renderer {
    pub(crate) gl: GL,
    pub(crate) cam: Camera,
    pub(crate) poly_mode: PolyMode,
    pub(crate) cull_face: Cull,
    pub(crate) bg_color: RGBA,
    pub(crate) msaa: bool,
    pub(crate) msaa_samples: u32,
    pub(crate) culling: bool,
}

impl Renderer {
    pub fn new() -> Result<Renderer, RendError> {
        let cam = Camera::new(Size2D::from(10, 10), CamProj::Ortho);
        let bg_color = RGBA::grey(0.5);
        let gl = match GL::load(10, 10) {
            Err(e) => return Err(RendError::GLError(e)),
            Ok(gl) => gl,
        };

        let mut renderer = Renderer {
            gl,
            cam,
            bg_color,
            msaa: true,
            culling: true,
            msaa_samples: 4,
            cull_face: Cull::AntiClock,
            poly_mode: PolyMode::Filled,
        };
        renderer.set_msaa(true);
        renderer.set_culling(true);
        renderer.set_wire_width(2.0);
        renderer.set_bg_color(bg_color);
        renderer.gl.enable_alpha(true);
        Ok(renderer)
    }
    pub(crate) fn set_size(&mut self, size: Size2D) {
        self.gl.resize(size);
    }
    fn clear(&self) {
        self.gl.clear()
    }

    pub fn set_msaa_samples(&mut self, samples: u32) {
        self.msaa_samples = samples
    }
    pub fn set_bg_color(&mut self, color: RGBA) {
        self.bg_color = color;
        self.gl.set_clear(color);
    }
    pub fn set_poly_mode(&mut self, mode: PolyMode) {
        self.poly_mode = mode;
        self.gl.poly_mode(mode);
    }
    pub fn toggle_wireframe(&mut self) {
        let new_poly_mode = match self.poly_mode {
            PolyMode::WireFrame => PolyMode::Filled,
            _ => PolyMode::WireFrame,
        };
        self.set_poly_mode(new_poly_mode);
    }
    pub fn set_msaa(&mut self, enable: bool) {
        self.msaa = enable;
        self.gl.enable_msaa(enable);
    }
    pub fn set_point_size(&self, size: f32) {
        self.gl.set_point_size(size)
    }
    pub fn toggle_msaa(&mut self) {
        self.msaa = !self.msaa;
        self.gl.enable_msaa(self.msaa)
    }
    pub fn set_culling(&mut self, enable: bool) {
        if self.culling != enable {
            self.toggle_culling()
        }
        self.gl.enable_cull(enable);
    }
    pub fn toggle_culling(&mut self) {
        self.culling = !self.culling;
        self.gl.enable_cull(self.culling);
    }
    pub fn set_cull_face(&mut self, cull_face: Cull) {
        self.cull_face = cull_face;
        self.gl.set_cull_face(self.cull_face)
    }
    pub fn flip_cull_face(&mut self) {
        self.cull_face = match self.cull_face {
            Cull::Clock => Cull::AntiClock,
            Cull::AntiClock => Cull::Clock,
        };
        self.gl.set_cull_face(self.cull_face);
    }
    pub fn set_wire_width(&mut self, width: f32) {
        self.gl.set_wire_width(width);
    }

    pub fn add_shader(&self, nshdr: ShaderFile) -> Result<Shader, RendError> {
        let id = match self.gl.create_src_program(&nshdr.v_src, &nshdr.f_src) {
            Ok(id) => id,
            Err(e) => {
                return Err(RendError::GLError(e));
            }
        };
        let mut shader = Shader::temporary();
        shader.id = id;
        Ok(shader)
    }
    pub fn remove_shader(&self, shader: Shader) {
        self.gl.delete_shader(shader.id)
    }

    pub fn add_texture(&self, ntxtr: TextureFile) -> Texture {
        let id = self.gl.create_texture(&ntxtr);

        Texture {
            id,
            size: ntxtr.size,
            fmt: ntxtr.fmt,
            filter: ntxtr.filter,
            wrap: ntxtr.wrap,
        }
    }
    pub fn remove_texture(&self, tex: Texture) {
        self.gl.delete_texture(tex.id)
    }

    pub(crate) fn create_mesh2d_handle(&self, nmesh: &Mesh2DFile) -> MeshHandle {
        let (vao_id, buf_id) = self.gl.create_buffer();
        let ind_id = self.gl.create_index_buffer();

        let (mut pos_info, mut pos_data) = (&ATTRInfo::empty(), &Vec::new());
        let (mut col_info, mut col_data) = (&ATTRInfo::empty(), &Vec::new());
        let (mut uvm_info, mut uvm_data) = (&ATTRInfo::empty(), &Vec::new());
        let (mut ind_info, mut ind_data) = (&ATTRInfo::empty(), &Vec::new());

        let mut cus_infos: Vec<&ATTRInfo> = Vec::new();
        let mut cus_datas: Vec<&Vec<u8>> = Vec::new();

        let (mut ind_count, mut vert_count, mut stride) = (0, 0, 0);

        let mut pos_exists = !nmesh.pos_attr.is_empty();
        let mut col_exists = !nmesh.col_attr.is_empty();
        let mut uvm_exists = !nmesh.uvm_attr.is_empty();

        if pos_exists {
            pos_info = &nmesh.pos_attr.info;
            pos_data = &nmesh.pos_attr.data;
            stride += pos_info.elem_count * pos_info.byte_count;
        }
        if col_exists {
            col_info = &nmesh.col_attr.info;
            col_data = &nmesh.col_attr.data;
            stride += col_info.elem_count * col_info.byte_count;
        }
        if uvm_exists {
            uvm_info = &nmesh.uvm_attr.info;
            uvm_data = &nmesh.uvm_attr.data;
            stride += uvm_info.elem_count * uvm_info.byte_count;
        }

        for cus_attr in nmesh.cus_attrs.iter() {
            let cus_info = &cus_attr.info;
            let cus_data = &cus_attr.data;
            stride += cus_info.elem_count * cus_info.byte_count;
            cus_infos.push(cus_info);
            cus_datas.push(cus_data);
        }

        let mut end = pos_data.len();
        if nmesh.starts_with_custom() {
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

            for (j, _attr) in nmesh.cus_attrs.iter().enumerate() {
                let cus_byte_count = cus_infos[j].byte_count * cus_infos[j].elem_count;
                let cus_data = cus_datas[j];
                let start = i * cus_byte_count;
                let end = ((i + 1) * (cus_byte_count)) - 1;
                buffer.push_attr(&cus_data[start..=end]);
            }
        }

        let mut attr_id = 0;
        let mut local_offset = 0;

        self.gl.bind_layouts(vao_id);
        self.gl.bind_buffer(buf_id);

        let mut layouts: Vec<(ATTRInfo, u32)> = Vec::new();
        if pos_exists {
            self.gl
                .set_attr_layout(&pos_info, attr_id, stride, local_offset);
            local_offset += pos_info.elem_count * pos_info.byte_count;
            layouts.push((pos_info.clone(), attr_id));
            attr_id += 1;
        }
        if col_exists {
            self.gl
                .set_attr_layout(&col_info, attr_id, stride, local_offset);
            local_offset += col_info.elem_count * col_info.byte_count;
            layouts.push((col_info.clone(), attr_id));
            attr_id += 1;
        }
        if uvm_exists {
            self.gl
                .set_attr_layout(&uvm_info, attr_id, stride, local_offset);
            local_offset += uvm_info.elem_count * uvm_info.byte_count;
            layouts.push((uvm_info.clone(), attr_id));
            attr_id += 1;
        }

        for cus_info in cus_infos.iter() {
            self.gl
                .set_attr_layout(cus_info, attr_id, stride, local_offset);
            local_offset += cus_info.elem_count * cus_info.byte_count;
            layouts.push((cus_info.deref().clone(), attr_id));
            attr_id += 1;
        }

        if buffer.len() > 0 {
            self.gl.fill_buffer(buf_id, &buffer);
        }
        self.gl.unbind_buffer();

        let mut has_indices = false;
        let mut index_buffer: Vec<u32> = Vec::new();

        if !nmesh.ind_attr.is_empty() {
            ind_info = &nmesh.ind_attr.info;
            ind_data = &nmesh.ind_attr.data;
            has_indices = true;
            for index in ind_data.iter() {
                ind_count += 1;
                index_buffer.push(*index);
            }
            self.gl.bind_index_buffer(ind_id);
            self.gl.fill_index_buffer(ind_id, &index_buffer);
            self.gl.unbind_index_buffer();
        }

        MeshHandle {
            layouts,
            has_indices,
            vert_count,
            ind_count,
            vao_id,
            buf_id,
            ind_id,
        }
    }

    pub(crate) fn create_mesh3d_handle(&self, nmesh: &Mesh3DFile) -> MeshHandle {
        let (vao_id, buf_id) = self.gl.create_buffer();
        let ind_id = self.gl.create_index_buffer();

        let (mut pos_info, mut pos_data) = (&ATTRInfo::empty(), &Vec::new());
        let (mut col_info, mut col_data) = (&ATTRInfo::empty(), &Vec::new());
        let (mut uvm_info, mut uvm_data) = (&ATTRInfo::empty(), &Vec::new());
        let (mut nrm_info, mut nrm_data) = (&ATTRInfo::empty(), &Vec::new());
        let (mut ind_info, mut ind_data) = (&ATTRInfo::empty(), &Vec::new());

        let mut cus_infos: Vec<&ATTRInfo> = Vec::new();
        let mut cus_datas: Vec<&Vec<u8>> = Vec::new();

        let (mut ind_count, mut vert_count, mut stride) = (0, 0, 0);

        let mut pos_exists = !nmesh.pos_attr.is_empty();
        let mut col_exists = !nmesh.col_attr.is_empty();
        let mut uvm_exists = !nmesh.uvm_attr.is_empty();
        let mut nrm_exists = !nmesh.nrm_attr.is_empty();

        if pos_exists {
            pos_info = &nmesh.pos_attr.info;
            pos_data = &nmesh.pos_attr.data;
            stride += pos_info.elem_count * pos_info.byte_count;
        }
        if col_exists {
            col_info = &nmesh.col_attr.info;
            col_data = &nmesh.col_attr.data;
            stride += col_info.elem_count * col_info.byte_count;
        }
        if uvm_exists {
            uvm_info = &nmesh.uvm_attr.info;
            uvm_data = &nmesh.uvm_attr.data;
            stride += uvm_info.elem_count * uvm_info.byte_count;
        }
        if nrm_exists {
            nrm_info = &nmesh.nrm_attr.info;
            nrm_data = &nmesh.nrm_attr.data;
            stride += nrm_info.elem_count * nrm_info.byte_count;
        }
        for cus_attr in nmesh.cus_attrs.iter() {
            let cus_info = &cus_attr.info;
            let cus_data = &cus_attr.data;
            stride += cus_info.elem_count * cus_info.byte_count;
            cus_infos.push(cus_info);
            cus_datas.push(cus_data);
        }
        let mut end = pos_data.len();
        if nmesh.starts_with_custom() {
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

            for (j, _attr) in nmesh.cus_attrs.iter().enumerate() {
                let cus_byte_count = cus_infos[j].byte_count * cus_infos[j].elem_count;
                let cus_data = cus_datas[j];
                let start = i * cus_byte_count;
                let end = ((i + 1) * (cus_byte_count)) - 1;
                buffer.push_attr(&cus_data[start..=end]);
            }
        }

        let mut attr_id = 0;
        let mut local_offset = 0;

        self.gl.bind_layouts(vao_id);
        self.gl.bind_buffer(buf_id);

        let mut layouts: Vec<(ATTRInfo, u32)> = Vec::new();
        if pos_exists {
            self.gl
                .set_attr_layout(&pos_info, attr_id, stride, local_offset);
            local_offset += pos_info.elem_count * pos_info.byte_count;
            layouts.push((pos_info.clone(), attr_id));
            attr_id += 1;
        }
        if col_exists {
            self.gl
                .set_attr_layout(&col_info, attr_id, stride, local_offset);
            local_offset += col_info.elem_count * col_info.byte_count;
            layouts.push((col_info.clone(), attr_id));
            attr_id += 1;
        }
        if uvm_exists {
            self.gl
                .set_attr_layout(&uvm_info, attr_id, stride, local_offset);
            local_offset += uvm_info.elem_count * uvm_info.byte_count;
            layouts.push((uvm_info.clone(), attr_id));
            attr_id += 1;
        }
        if nrm_exists {
            self.gl
                .set_attr_layout(&nrm_info, attr_id, stride, local_offset);
            local_offset += nrm_info.elem_count * nrm_info.byte_count;
            layouts.push((nrm_info.clone(), attr_id));
            attr_id += 1;
        }

        for cus_info in cus_infos.iter() {
            self.gl
                .set_attr_layout(cus_info, attr_id, stride, local_offset);
            local_offset += cus_info.elem_count * cus_info.byte_count;
            layouts.push((cus_info.deref().clone(), attr_id));
            attr_id += 1;
        }

        if buffer.len() > 0 {
            self.gl.fill_buffer(buf_id, &buffer);
        }
        self.gl.unbind_buffer();

        let mut has_indices = false;
        let mut index_buffer: Vec<u32> = Vec::new();

        if !nmesh.ind_attr.is_empty() {
            ind_info = &nmesh.ind_attr.info;
            ind_data = &nmesh.ind_attr.data;
            has_indices = true;
            for index in ind_data.iter() {
                ind_count += 1;
                index_buffer.push(*index);
            }
            self.gl.bind_index_buffer(ind_id);
            self.gl.fill_index_buffer(ind_id, &index_buffer);
            self.gl.unbind_index_buffer();
        }

        MeshHandle {
            layouts,
            has_indices,
            vert_count,
            ind_count,
            vao_id,
            buf_id,
            ind_id,
        }
    }

    pub fn add_mesh3d(&self, nmesh: Mesh3DFile) -> NEMesh3D {
        let handle = self.create_mesh3d_handle(&nmesh);
        NEMesh3D {
            handle,
            visibility: true,
            shader: None,
            transform: Transform3D::default(),
            draw_mode: DrawMode::default(),
        }
    }
    pub fn remove_mesh3d(&self, mesh: NEMesh3D) {
        drop(mesh)
    }

    pub fn add_mesh2d(&self, nmesh: Mesh2DFile) -> NEMesh2D {
        let handle = self.create_mesh2d_handle(&nmesh);
        NEMesh2D {
            handle,
            visibility: true,
            shader: None,
            transform: Transform2D::default(),
            draw_mode: DrawMode::default(),
        }
    }
    pub fn remove_mesh2d(&self, mesh: NEMesh2D) {
        drop(mesh)
    }

    pub fn render3d(&self, mesh: &NEMesh3D) {
        if !mesh.is_visible() {
            return;
        }
        let shader = match &mesh.shader {
            None => return,
            Some(sh) => sh,
        };
        let s = shader.id;
        let handle = &mesh.handle;

        let tfm = mesh.transform.matrix();

        self.gl.bind_shader(s);
        self.gl
            .set_uni_m4f32(s, "uView", self.cam.transform.view_matrix());
        self.gl
            .set_uni_m4f32(s, "uProj", self.cam.transform.proj_matrix());
        self.gl.set_uni_m4f32(s, "uTfm", tfm);

        self.bind_textures(&shader.tex_ids);
        self.draw(handle, &mesh.draw_mode)
    }
    pub fn render2d(&self, mesh: &NEMesh2D) {
        if !mesh.is_visible() {
            return;
        }
        let shader = match &mesh.shader {
            None => return,
            Some(sh) => sh,
        };
        let s = shader.id;
        let handle = &mesh.handle;

        let scale = 1.0;
        let max_layers = 255;
        let tfm = mesh.transform.matrix();
        let layer = mesh.transform.layer() as u32;
        let w = self.cam.transform.size.aspect_ratio() * scale;
        let proj = ortho(-w, w, -scale, scale, 0.0, -(max_layers + 1) as f32);

        self.gl.bind_shader(s);
        self.gl.set_uni_m4f32(s, "uProj", proj);
        self.gl.set_uni_m4f32(s, "uTfm", tfm);
        self.gl.set_uni_u32(s, "uLayer", layer);

        self.bind_textures(&shader.tex_ids);
        self.draw(handle, &mesh.draw_mode)
    }

    fn bind_textures(&self, tex_ids: &Vec<Option<u32>>) {
        for (slot, tex_id) in tex_ids.iter().enumerate() {
            match tex_id {
                None => {}
                Some(id) => {
                    self.gl.bind_texture_at(*id, slot as u32);
                }
            }
        }
    }

    fn draw(&self, handle: &MeshHandle, draw_mode: &DrawMode) {
        self.gl.bind_layouts(handle.vao_id);
        match handle.has_indices {
            false => self.gl.draw_array(&draw_mode, handle.vert_count),
            true => {
                self.gl.bind_index_buffer(handle.ind_id);
                self.gl.draw_indexed(&draw_mode, handle.ind_count);
            }
        }
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

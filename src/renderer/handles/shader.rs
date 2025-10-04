use crate::Texture;

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
      }
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
}

impl Shader {
   pub fn attach_tex(&mut self, tex: &Texture) {
      for (slot, tex_id) in self.tex_ids.iter().enumerate() {
         match tex_id {
            None => {
               self.tex_ids[slot] = Some(tex.id);
               break;
            }
            Some(_) => {}
         }
      }
   }

   pub fn set_tex_at_slot(&mut self, tex: &Texture, slot: TexSlot) {
      self.tex_ids[slot.as_index()] = Some(tex.id)
   }
}

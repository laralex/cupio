use std::{io::Cursor, ffi::{CStr}};

use ash::{vk::{self, ShaderModule}, util::read_spv};

use super::VulkanDrop;

const MAX_SHADER_STAGES: usize = 4;

use lazy_static::lazy_static;

lazy_static! {
   static ref SHADER_ENTRY_FUNCTION_NAME: &'static CStr = unsafe {
      CStr::from_bytes_with_nul_unchecked(b"main\0")
   };
}
pub struct VulkanShader {
   shader_stage_create_infos: [vk::PipelineShaderStageCreateInfo; MAX_SHADER_STAGES],
   n_stages: usize,
}

impl VulkanShader {
   pub fn builder(device: &ash::Device) -> VulkanShaderBuilder {
      VulkanShaderBuilder {
         shader_stage_create_infos: Default::default(),
         n_stages: 0,
         device
      }
   }

   pub fn shader_stage_create_infos(&self) -> &[vk::PipelineShaderStageCreateInfo]{
      &self.shader_stage_create_infos[..self.n_stages]
   }

   pub fn n_stages(&self) -> usize {
      self.n_stages
   }

   pub fn put_into(&mut self, stage_idx: usize, info: vk::PipelineShaderStageCreateInfo) {
      assert!(stage_idx < usize::min(MAX_SHADER_STAGES, self.n_stages+1), 
         "Cannot leave gaps in the array of stages, use `stage_idx` at most `n_stages`");
      self.shader_stage_create_infos[stage_idx] = info;
      self.n_stages = usize::max(self.n_stages, stage_idx + 1);
   }
}

impl VulkanDrop for VulkanShader {
    fn drop(self, device: &ash::Device) {
      self.shader_stage_create_infos.iter()
         .map(|info| info.module)
         .for_each(|shader_module| unsafe {
            device.destroy_shader_module(shader_module, None);
         })
    }
}
pub struct VulkanShaderBuilder<'a> {
   shader_stage_create_infos: [vk::PipelineShaderStageCreateInfo; MAX_SHADER_STAGES],
   n_stages: usize,
   device: &'a ash::Device,
}

impl<'a> VulkanShaderBuilder<'a>{
   pub fn build(self) -> VulkanShader {
      assert!(self.n_stages > 0, "Must initialize atleast 1 shader stage");
      let all_infos_intialized = self.shader_stage_create_infos.iter()
         .take(self.n_stages)
         .find(|&info| info.module == Default::default())
         .is_none();
      assert!(all_infos_intialized, "If you initialize stage_idx==N, all stages {{0,1,...,N}} must be initialized at some point");
      VulkanShader {
         shader_stage_create_infos: self.shader_stage_create_infos,
         n_stages: self.n_stages,
      }
   }

   pub fn with_vertex_shader_file(self, stage_idx: usize, shader_spv_file: &mut Cursor<impl AsRef<[u8]>>) -> Self {
      Self::check_stage_idx(stage_idx);
      let shader_module = self.make_shader_module(shader_spv_file);
      self.with_vertex_shader(stage_idx, shader_module)
   }

   pub fn with_vertex_shader(mut self, stage_idx: usize, shader_module: ShaderModule) -> Self {
      Self::check_stage_idx(stage_idx);
      self.put_info_unchecked(stage_idx, vk::PipelineShaderStageCreateInfo {
         module: shader_module,
         p_name: SHADER_ENTRY_FUNCTION_NAME.as_ptr(),
         stage: vk::ShaderStageFlags::VERTEX,
         ..Default::default()
      });
      self
   }

   pub fn with_fragment_shader_file(self, stage_idx: usize, shader_spv_file: &mut Cursor<impl AsRef<[u8]>>) -> Self {
      Self::check_stage_idx(stage_idx);
      let shader_module = self.make_shader_module(shader_spv_file);
      self.with_fragment_shader(stage_idx, shader_module)
   }

   pub fn with_fragment_shader(mut self, stage_idx: usize, shader_module: ShaderModule) -> Self {
      Self::check_stage_idx(stage_idx);
      self.put_info_unchecked(stage_idx, vk::PipelineShaderStageCreateInfo {
          s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
          module: shader_module,
          p_name: SHADER_ENTRY_FUNCTION_NAME.as_ptr(),
          stage: vk::ShaderStageFlags::FRAGMENT,
          ..Default::default()
      });
      self
   }

   fn make_shader_module(&self, shader_spv_file: &mut Cursor<impl AsRef<[u8]>>) -> ShaderModule {
      let code = read_spv(shader_spv_file)
         .expect("Failed to read fragment shader spv file");
      let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);
      unsafe {
         self.device
            .create_shader_module(&shader_info, None)
            .expect("Shader module error")
      }
   }

   fn put_info_unchecked(&mut self, stage_idx: usize, info: vk::PipelineShaderStageCreateInfo) {
      self.shader_stage_create_infos[stage_idx] = info;
      self.n_stages = usize::max(self.n_stages, stage_idx + 1);
   }

   fn check_stage_idx(stage_idx: usize) {
      assert!(stage_idx < MAX_SHADER_STAGES, "Requested more shader stages than supported");
   }
}
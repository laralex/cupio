
use std::marker::PhantomData;

use ash::vk;

pub struct VulkanVertices<VertexT: Sized> {
    input_binding_descriptions: [vk::VertexInputBindingDescription; 1],
    input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
    next_shader_location: u32,
    input_assembly_state_create_info: vk::PipelineInputAssemblyStateCreateInfo,
    _phantom: PhantomData<VertexT>,
}

impl<VertexT> VulkanVertices<VertexT> {
    pub fn new_vertex_data() -> Self {
        Self::new(vk::VertexInputRate::VERTEX)
    }

    pub fn new_instanced_data() -> Self {
        Self::new(vk::VertexInputRate::INSTANCE)
    }

    pub fn with_topology(mut self, topology: vk::PrimitiveTopology) -> Self {
        self.input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology,
            ..Default::default()
        };
        self
    }

    pub fn add_vec4_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32G32B32A32_SFLOAT, offset)
    }

    pub fn add_vec3_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32G32B32_SFLOAT, offset)
    }

    pub fn add_vec2_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32G32_SFLOAT, offset)
    }

    pub fn add_float_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32_SFLOAT, offset)
    }

    pub fn add_int_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32_UINT, offset)
    }

    pub fn add_ivec2_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32G32_UINT, offset)
    }

    pub fn add_ivec3_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32G32B32_UINT, offset)
    }

    pub fn add_ivec4_attribute(self, offset: u32) -> Self {
        self.add_attribute(vk::Format::R32G32B32A32_UINT, offset)
    }

    pub fn add_attribute(mut self, format: vk::Format, offset: u32) -> Self{
        self.input_attribute_descriptions.push(
            vk::VertexInputAttributeDescription {
                location: self.next_shader_location,
                binding: 0,
                format,
                offset,
            }
        );
        self.next_shader_location += 1;
        self
    }

    pub fn get_vertex_input_state(&self) -> vk::PipelineVertexInputStateCreateInfo {
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&self.input_attribute_descriptions)
            .vertex_binding_descriptions(&self.input_binding_descriptions)
            .build()
    }

    pub fn get_input_assembly_state(&self) -> &vk::PipelineInputAssemblyStateCreateInfo {
        &self.input_assembly_state_create_info
    }

    fn new(input_rate: vk::VertexInputRate) -> Self {
        let input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<VertexT>() as u32,
            input_rate,
        }];
        Self {
            input_binding_descriptions,
            input_attribute_descriptions: vec![],
            input_assembly_state_create_info: Default::default(),
            next_shader_location: 0,
            _phantom: PhantomData
        }
    }
}
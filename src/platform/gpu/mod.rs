pub mod abstraction;
pub mod vulkan_context; // TODO: make private
pub mod vulkan_shader;
pub mod vulkan_vertices;

pub trait VulkanDrop {
   fn drop(self, device: &ash::Device);
}
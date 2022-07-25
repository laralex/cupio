use std::{marker::PhantomData, mem::align_of};

use ash::{vk::{self, SharingMode}, util::Align};

use super::{vulkan_context::find_memorytype_index, VulkanDrop};

pub struct VulkanBuffer<InnerT>{
   buffer_memory: vk::DeviceMemory,
   buffer: vk::Buffer,
   len: usize,
   _phantom: PhantomData<InnerT>,
}

impl<InnerT: Copy> VulkanBuffer<InnerT> {
   pub fn builder<'a>(device: &'a ash::Device, device_memory_properties: &'a vk::PhysicalDeviceMemoryProperties) -> VulkanBufferBuilder<'a, InnerT> {
      VulkanBufferBuilder::new(device, device_memory_properties)
   }

   pub fn get_buffer(&self) -> &vk::Buffer {
      &self.buffer
   }

   pub fn len(&self) -> usize {
      self.len
   }
}

impl<InnerT> VulkanDrop for VulkanBuffer<InnerT>{
   fn drop(self, device: &ash::Device) {
      unsafe {
         device.free_memory(self.buffer_memory, None);
         device.destroy_buffer(self.buffer, None);
      }
   }
}

pub struct VulkanBufferBuilder<'a, InnerT> {
   device: &'a ash::Device,
   device_memory_properties: &'a vk::PhysicalDeviceMemoryProperties,
   usage: Option<vk::BufferUsageFlags>,
   sharing_mode: Option<vk::SharingMode>,
   create_flags: vk::BufferCreateFlags,
   _phantom: PhantomData<InnerT>,
}

impl<'a, InnerT: Copy> VulkanBufferBuilder<'a, InnerT> {
   pub fn new(device: &'a ash::Device, device_memory_properties: &'a vk::PhysicalDeviceMemoryProperties) -> Self {
      Self {
         device,
         device_memory_properties,
         create_flags: vk::BufferCreateFlags::default(),
         sharing_mode: None,
         usage: None,
         _phantom: PhantomData,
      }
   }

   pub fn exclusive(mut self) -> Self {
      self.sharing_mode = Some(SharingMode::EXCLUSIVE);
      self
   }

   pub fn concurrent(self) -> Self {
      panic!("Untested, add queues indices");
      self.sharing_mode = Some(SharingMode::CONCURRENT);
      self
   }

   pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
      self.usage = Some(usage);
      self
   }

   pub fn sparse_binding(mut self) -> Self {
      panic!("Untested");
      self.create_flags |= vk::BufferCreateFlags::SPARSE_BINDING;
      self
   }

   pub fn sparse_residency(mut self) -> Self {
      panic!("Untested");
      self.create_flags |= vk::BufferCreateFlags::SPARSE_BINDING | vk::BufferCreateFlags::SPARSE_BINDING;
      self
   }

   pub fn build(self, content: &[InnerT]) -> VulkanBuffer<InnerT> {
      assert!(self.sharing_mode.is_some() && self.usage.is_some(), "Must specify buffer usage via .usage() and sharing mode via .exclusive() or .concurrent()");
      let buffer_info = vk::BufferCreateInfo {
         size: (content.len() * std::mem::size_of::<InnerT>()) as u64,
         usage: self.usage.unwrap(),
         sharing_mode: self.sharing_mode.unwrap(),
         flags: self.create_flags,
         ..Default::default()
      };

      let buffer;
      let buffer_memory_req;
      unsafe {
         buffer = self.device
            .create_buffer(&buffer_info, None)
            .unwrap();

         buffer_memory_req = self.device
            .get_buffer_memory_requirements(buffer);

      }

      let vertex_input_buffer_memory_index = find_memorytype_index(
         &buffer_memory_req,
         self.device_memory_properties,
         vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
      )
      .expect("Unable to find suitable memorytype for the vertex buffer.");

      let memory_allocate_info = vk::MemoryAllocateInfo {
         allocation_size: buffer_memory_req.size,
         memory_type_index: vertex_input_buffer_memory_index,
         ..Default::default()
      };

      let buffer_memory;
      unsafe {
         buffer_memory = self.device
            .allocate_memory(&memory_allocate_info, None)
            .unwrap();
         let mapped_memory_ptr = self.device
            .map_memory(
                  buffer_memory,
                  0,
                  buffer_memory_req.size,
                  vk::MemoryMapFlags::empty(),
            )
            .unwrap();

         let mut buffer_align = Align::new(
            mapped_memory_ptr,
            align_of::<InnerT>() as u64,
            buffer_memory_req.size,
         );

         buffer_align.copy_from_slice(&content); // safe

         self.device.unmap_memory(buffer_memory);
         self.device
            .bind_buffer_memory(buffer, buffer_memory, 0)
            .unwrap();
      }
      VulkanBuffer {
         buffer,
         buffer_memory,
         len: content.len(),
         _phantom: PhantomData,
      }
   }
}
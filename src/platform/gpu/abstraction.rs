
use std::ops::Drop;

use super::vulkan_context::VulkanContext;

pub struct GpuContextDispatcher {
  api_context: ApiContext,
}

impl Drop for GpuContextDispatcher {
  fn drop(&mut self) {
      use ApiContext::*;
      match &mut self.api_context {
        Vulkan(ctx) => drop(ctx),
        OpenGl(ctx) => (),
      }
  }
}

enum ApiContext {
  Vulkan(VulkanContext),
  OpenGl(()),
}



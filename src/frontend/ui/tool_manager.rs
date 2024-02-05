use std::collections::HashMap;

use super::tool::{crop::Crop, Tool, ToolIdentifier};

pub struct ToolManager {
    tools: HashMap<ToolIdentifier, Box<dyn Tool>>,
    active_tool: Option<ToolIdentifier>,
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: Self::create_tools(),
            active_tool: Some(ToolIdentifier::Crop),
        }
    }

    fn create_tools() -> HashMap<ToolIdentifier, Box<dyn Tool>> {
        HashMap::from([(ToolIdentifier::Crop, Crop::boxed() as Box<dyn Tool>)])
    }

    pub fn active_tool(&self) -> Option<&dyn Tool> {
        if let Some(identifer) = &self.active_tool {
            return Some(
                self.tools
                    .get(identifer)
                    .expect("Couldn't find tool instance. This is likely a bug.")
                    .as_ref(),
            );
        }

        None
    }

    pub fn active_tool_mut(&mut self) -> Option<&mut dyn Tool> {
        if let Some(identifer) = &self.active_tool {
            return Some(
                self.tools
                    .get_mut(identifer)
                    .expect("Couldn't find tool instance. This is likely a bug.")
                    .as_mut(),
            );
        }

        None
    }

    pub fn set_active_tool(&mut self, identifier: Option<ToolIdentifier>) {
        self.active_tool = identifier;
    }
}

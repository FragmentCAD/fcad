use fcad_core::application::tools::ToolManager;
use fcad_core::application::tools::space_tool::SpaceTool;
use fcad_core::application::tools::line_tool::LineTool;
use fcad_core::application::tools::rect_tool::RectTool;
use fcad_core::application::tools::erase_tool::EraseTool;

pub struct ToolService;

impl ToolService {
    pub fn set_tool(tm: &mut ToolManager, tool_name: &str) -> String {
        match tool_name {
            "space" => {
                tm.set_tool(Box::new(SpaceTool::new()));
                "space".to_string()
            }
            "line" => {
                tm.set_tool(Box::new(LineTool::new()));
                "line".to_string()
            }
            "rect" => {
                tm.set_tool(Box::new(RectTool::new()));
                "rect".to_string()
            }
            "erase" => {
                tm.set_tool(Box::new(EraseTool::new()));
                "erase".to_string()
            }
            "none" | "" => {
                tm.clear_tool();
                "none".to_string()
            }
            other => format!("unknown tool: {}", other),
        }
    }

    pub fn get_active_tool(tm: &ToolManager) -> String {
        tm.active_tool_name().unwrap_or("none").to_string()
    }
}

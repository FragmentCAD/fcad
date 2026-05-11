use fcad_core::application::tools::erase_tool::EraseTool;
use fcad_core::application::tools::line_tool::LineTool;
use fcad_core::application::tools::rect_tool::RectTool;
use fcad_core::application::tools::space_tool::SpaceTool;
use fcad_core::application::tools::ToolManager;

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

#[cfg(test)]
mod tests {
    use super::*;
    use fcad_core::application::tools::ToolManager;

    #[test]
    fn test_set_tool() {
        let mut tm = ToolManager::new();
        let result = ToolService::set_tool(&mut tm, "line");
        assert_eq!(result, "line");
        assert_eq!(ToolService::get_active_tool(&tm), "line");
    }

    #[test]
    fn test_clear_tool() {
        let mut tm = ToolManager::new();
        ToolService::set_tool(&mut tm, "line");
        ToolService::set_tool(&mut tm, "none");
        assert_eq!(ToolService::get_active_tool(&tm), "none");
    }

    #[test]
    fn test_unknown_tool() {
        let mut tm = ToolManager::new();
        let result = ToolService::set_tool(&mut tm, "invalid");
        assert!(result.contains("unknown"));
    }
}
